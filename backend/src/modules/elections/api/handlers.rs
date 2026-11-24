//! Handlers do quadro público, do recibo e do cadastro da conta autenticada.
//!
//! Esta fronteira extrai dados HTTP e monta a resposta de cada operação. As
//! rotas e os limites de requisição são compostos no módulo de API.

use axum::{
    Json,
    body::Body,
    extract::{Path, State, rejection::JsonRejection},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::IntoResponse,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    api::{
        error::{ApiError, ApiResult},
        state::AppState,
    },
    modules::{
        elections::{
            api::{
                dto::{
                    AuthorityRegistrationRequestDto, BallotRequestDto, CancellationRequest,
                    PeriodRequest, RegistrationRequestDto, RegistrationResponse, ShareRequestDto,
                },
                error::{
                    ballot_error, public_read_error, receipt_not_found, receipt_read_error,
                    signed_request_error, tally_error,
                },
            },
            application::{
                administration::{BoardCancellation, UpdatedPeriod},
                board::BoardError,
                preparation::AuthorityAssignment,
                registration::{RegistrationStatus, VoterIdentity},
                services::{administration, board, preparation, tally, voter_registration},
                tally::TallyEvidence,
            },
            domain::{
                entities::{election::ElectionPeriod, vote::VoteKey},
                identifiers::{AccountId, AuthorityId, ElectionId},
            },
            infrastructure::{RsaPssBallotVerifier, RsaPssVoterKeyProofVerifier},
        },
        identity::api::extractors::AuthenticatedAccount,
    },
};

pub(super) async fn snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let chunks = board::stream_snapshot(&state.pg_database, route_election_id(id)?)
        .await
        .map_err(public_read_error)?;

    Ok((
        [(CONTENT_TYPE, "application/json")],
        Body::from_stream(chunks),
    ))
}

pub(super) async fn checkpoint(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    board::read_checkpoint(&state.pg_database, route_election_id(id)?)
        .await
        .map(Json)
        .map_err(public_read_error)
}

pub(super) async fn read_own_receipt(
    State(state): State<AppState>,
    Path(election_id): Path<Uuid>,
    account: AuthenticatedAccount,
) -> ApiResult<Json<Value>> {
    let key = VoteKey::new(
        route_election_id(election_id)?,
        authenticated_account_id(account.account_id)?,
    );

    let receipt = board::read_own_receipt(&state.pg_database, key)
        .await
        .map_err(receipt_read_error)?
        .ok_or_else(receipt_not_found)?;

    Ok(Json(receipt))
}

pub(super) async fn read_own_registration(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    let result = voter_registration::status(
        &state.pg_database,
        route_election_id(election_id)?,
        authenticated_account_id(account.account_id)?,
    )
    .await?;
    Ok(registration_response(result))
}

pub(super) async fn register_voter_key(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
    body: Result<Json<RegistrationRequestDto>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(request) = body.map_err(|error| {
        signed_request_error(
            error,
            "invalid_registration_proof",
            "Prova de cadastro inválida.",
        )
    })?;
    let result = voter_registration::register(
        &state.pg_database,
        &RsaPssVoterKeyProofVerifier,
        route_election_id(election_id)?,
        voter_identity(account)?,
        request.into_proof(),
    )
    .await?;
    Ok(registration_response(result))
}

pub(super) async fn submit_ballot(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election_id): Path<Uuid>,
    body: Result<Json<BallotRequestDto>, JsonRejection>,
) -> ApiResult<impl IntoResponse> {
    let Json(request) = body.map_err(|error| {
        signed_request_error(
            error,
            "invalid_ballot",
            "Cédula inválida para a conta e a eleição.",
        )
    })?;
    let election_id = route_election_id(election_id)?;
    let signer = state
        .election_signer
        .as_deref()
        .ok_or_else(ApiError::service_unavailable)?;
    let identity = voter_identity(account)?;
    let ballot = request
        .into_signed_ballot(election_id, identity.account_id)
        .map_err(|error| ballot_error(BoardError::InvalidEvidence(Box::new(error))))?;

    let recorded = board::record_ballot(
        &RsaPssBallotVerifier,
        &state.pg_database,
        signer,
        identity,
        ballot,
    )
    .await
    .map_err(ballot_error)?;
    let status = if recorded.recovered {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    Ok((status, [(CACHE_CONTROL, "no-store")], Json(recorded)))
}

pub(super) async fn tally_result(
    State(state): State<AppState>,
    Path(election): Path<Uuid>,
) -> ApiResult<Json<TallyEvidence>> {
    tally::read_result(&state.pg_database, route_election_id(election)?)
        .await
        .map(Json)
        .map_err(tally_error)
}

pub(super) async fn submit_share(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path((election, authority)): Path<(Uuid, Uuid)>,
    body: Result<Json<ShareRequestDto>, JsonRejection>,
) -> ApiResult<Json<TallyEvidence>> {
    let Json(request) = body.map_err(|error| {
        signed_request_error(
            error,
            "invalid_tally_evidence",
            "A evidência não confere com a apuração.",
        )
    })?;
    let signer = state
        .election_signer
        .as_deref()
        .ok_or_else(ApiError::service_unavailable)?;
    let submission = request
        .into_submission()
        .map_err(|error| tally_error(BoardError::InvalidEvidence(Box::new(error))))?;
    let election = route_election_id(election)?;
    let authority = AuthorityId::try_from_uuid(authority).map_err(|_| ApiError::bad_request())?;
    let account = authenticated_account_id(account.account_id)?;

    tally::submit_share(
        &state.pg_database,
        &state.pg_database,
        signer,
        election,
        authority,
        account,
        submission,
    )
    .await
    .map(Json)
    .map_err(tally_error)
}

pub(super) async fn own_authority(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election): Path<Uuid>,
) -> ApiResult<Json<AuthorityAssignment>> {
    preparation::assignment(
        &state.pg_database,
        route_election_id(election)?,
        authenticated_account_id(account.account_id)?,
    )
    .await
    .map(Json)
    .map_err(tally_error)
}

pub(super) async fn register_authority(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election): Path<Uuid>,
    body: Result<Json<AuthorityRegistrationRequestDto>, JsonRejection>,
) -> ApiResult<Json<AuthorityAssignment>> {
    let Json(request) = body.map_err(|error| {
        signed_request_error(
            error,
            "invalid_tally_evidence",
            "A evidência não confere com a apuração.",
        )
    })?;
    let signer = state
        .election_signer
        .as_deref()
        .ok_or_else(ApiError::service_unavailable)?;
    let proof = request
        .into_proof()
        .map_err(|error| tally_error(BoardError::InvalidEvidence(Box::new(error))))?;

    preparation::register_authority(
        &state.pg_database,
        signer,
        route_election_id(election)?,
        authenticated_account_id(account.account_id)?,
        &proof,
    )
    .await
    .map(Json)
    .map_err(tally_error)
}

pub(super) async fn publish_manifest(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election): Path<Uuid>,
) -> ApiResult<Json<AuthorityAssignment>> {
    let signer = state
        .election_signer
        .as_deref()
        .ok_or_else(ApiError::service_unavailable)?;
    preparation::publish_manifest(
        &state.pg_database,
        signer,
        route_election_id(election)?,
        authenticated_account_id(account.account_id)?,
    )
    .await
    .map(Json)
    .map_err(tally_error)
}

pub(super) async fn reschedule(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election): Path<Uuid>,
    Json(request): Json<PeriodRequest>,
) -> ApiResult<Json<UpdatedPeriod>> {
    let period = ElectionPeriod::try_new(request.starts_at, request.ends_at)
        .map_err(|_| tally_error(BoardError::InvalidAdministration("período inválido")))?;
    administration::reschedule(
        &state.pg_database,
        route_election_id(election)?,
        authenticated_account_id(account.account_id)?,
        period,
        &request.reason,
    )
    .await
    .map(Json)
    .map_err(tally_error)
}

pub(super) async fn cancel(
    State(state): State<AppState>,
    account: AuthenticatedAccount,
    Path(election): Path<Uuid>,
    Json(request): Json<CancellationRequest>,
) -> ApiResult<Json<BoardCancellation>> {
    administration::cancel(
        &state.pg_database,
        route_election_id(election)?,
        authenticated_account_id(account.account_id)?,
        &request.reason,
    )
    .await
    .map(Json)
    .map_err(tally_error)
}

fn route_election_id(id: Uuid) -> ApiResult<ElectionId> {
    ElectionId::try_from_uuid(id).map_err(|_| ApiError::bad_request())
}

fn authenticated_account_id(id: Uuid) -> ApiResult<AccountId> {
    AccountId::try_from_uuid(id).map_err(|_| ApiError::internal_server_error())
}

fn voter_identity(account: AuthenticatedAccount) -> ApiResult<VoterIdentity> {
    Ok(VoterIdentity {
        account_id: authenticated_account_id(account.account_id)?,
        microsoft_object_id: account.microsoft_object_id,
    })
}

fn registration_response(status: RegistrationStatus) -> impl IntoResponse {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(RegistrationResponse::from(status)),
    )
}
