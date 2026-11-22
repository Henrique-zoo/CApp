//! Manifesto imutável que define a configuração de uma eleição.

use std::collections::HashSet;

use crypto_bigint::U2048;

use crate::modules::elections::domain::{
    Fingerprint,
    choices::{ElectionOption, ElectionOptions},
    entities::authority::{Authority, AuthorityRole},
    error::{InvalidElectionOptions, InvalidManifest},
    identifiers::{AcademicCenterId, ElectionId},
};

/// Identificador do grupo criptográfico definido pelo protocolo v1.
pub const ELECTION_GROUP_ID: &str = "rfc3526-modp14";

// Preserva o caminho público usado pelos consumidores do manifesto.
pub use crate::modules::elections::domain::profile::{
    MAX_AUTHORITIES, MAX_OPTIONS, MIN_AUTHORITIES, MIN_OPTIONS,
};

/// Versão do protocolo eleitoral.
pub const MANIFEST_VERSION: u8 = 1;

/// Manifesto cuja criptografia e digest foram conferidos pelo adaptador.
///
/// A implementação de `ElectionCryptography` verifica grupo, chaves, provas,
/// chave conjunta e a chave RSA; o codec calcula/confere o digest. Somente
/// depois ela constrói esta evidência. A reconstituição do armazenamento deve
/// passar pela mesma fronteira. O domínio não implementa essas verificações.
///
/// O construtor restrito ao crate evita que consumidores externos tratem um
/// manifesto apenas estrutural como autorização para abrir uma eleição.
///
/// ```compile_fail
/// use backend::modules::elections::domain::{
///     Fingerprint,
///     entities::manifest::{Manifest, VerifiedManifest},
/// };
/// fn bypass(manifest: Manifest, hash: Fingerprint) -> VerifiedManifest {
///     VerifiedManifest::from_verified_parts(manifest, hash)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedManifest {
    manifest: Manifest,
    hash: Fingerprint,
}

impl VerifiedManifest {
    /// Registra a verificação realizada pelas portas de criptografia e codec.
    #[allow(
        dead_code,
        reason = "A implementação criptográfica pertence às issues #378–#380; testes usam evidências controladas."
    )]
    pub(crate) fn from_verified_parts(manifest: Manifest, hash: Fingerprint) -> Self {
        Self { manifest, hash }
    }

    /// Retorna a configuração pública verificada.
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Retorna o digest calculado ou conferido pelo codec.
    pub const fn hash(&self) -> &Fingerprint {
        &self.hash
    }
}

/// Manifesto imutável de uma eleição.
///
/// A ordem das opções e das autoridades é preservada, pois faz parte
/// da configuração publicada pelo protocolo. A referência ao calendário inicial
/// pertence ao envelope assinado que o adaptador publica junto da configuração.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    election_id: ElectionId,
    academic_center_id: AcademicCenterId,
    options: ElectionOptions,
    authorities: Vec<Authority>,
    combined_public_key: U2048,
    server_signing_key: Vec<u8>,
}

impl Manifest {
    /// Constrói um manifesto validando suas invariantes estruturais.
    pub fn try_new(
        election_id: ElectionId,
        academic_center_id: AcademicCenterId,
        options: Vec<ElectionOption>,
        authorities: Vec<Authority>,
        combined_public_key: U2048,
        server_signing_key: Vec<u8>,
    ) -> Result<Self, InvalidManifest> {
        let options = ElectionOptions::try_new(options).map_err(|error| match error {
            InvalidElectionOptions::InvalidCount => InvalidManifest::InvalidOptionCount,
            InvalidElectionOptions::DuplicateId => InvalidManifest::DuplicateOptionId,
            InvalidElectionOptions::InvalidNullCount => InvalidManifest::InvalidNullOptionCount,
        })?;
        if !matches!(options.as_slice().last(), Some(ElectionOption::Null)) {
            return Err(InvalidManifest::NullOptionMustBeLast);
        }

        if !(MIN_AUTHORITIES..=MAX_AUTHORITIES).contains(&authorities.len()) {
            return Err(InvalidManifest::InvalidAuthorityCount);
        }

        if combined_public_key <= U2048::ONE {
            return Err(InvalidManifest::InvalidCombinedPublicKey);
        }

        let slate_ids: HashSet<_> = options
            .as_slice()
            .iter()
            .filter_map(ElectionOption::slate_id)
            .collect();

        let mut authority_ids = HashSet::with_capacity(authorities.len());
        let mut signing_keys = HashSet::with_capacity(authorities.len());
        let mut public_values = HashSet::with_capacity(authorities.len());
        let mut authority_slate_ids = HashSet::with_capacity(authorities.len());

        let mut server_authority_count = 0usize;
        let mut commission_authority_count = 0usize;
        for authority in &authorities {
            if !authority_ids.insert(*authority.id().as_uuid()) {
                return Err(InvalidManifest::DuplicateAuthorityId);
            }

            if !signing_keys.insert(authority.signing_key()) {
                return Err(InvalidManifest::DuplicateAuthoritySigningKey);
            }
            if !public_values.insert(authority.public_value()) {
                return Err(InvalidManifest::DuplicateAuthorityPublicValue);
            }

            match authority.role() {
                AuthorityRole::Server => {
                    server_authority_count += 1;
                }
                AuthorityRole::Commission => {
                    commission_authority_count += 1;
                }
                AuthorityRole::Slate(slate_id) => {
                    if !authority_slate_ids.insert(slate_id) {
                        return Err(InvalidManifest::SlateAuthorityMismatch);
                    }
                }
            }
        }

        if server_authority_count != 1 {
            return Err(InvalidManifest::InvalidServerAuthorityCount);
        }

        if commission_authority_count != 1 {
            return Err(InvalidManifest::InvalidCommissionAuthorityCount);
        }

        if authority_slate_ids.len() != slate_ids.len()
            || !authority_slate_ids.is_subset(&slate_ids)
        {
            return Err(InvalidManifest::SlateAuthorityMismatch);
        }

        Ok(Self {
            election_id,
            academic_center_id,
            options,
            authorities,
            combined_public_key,
            server_signing_key,
        })
    }

    /// Retorna o identificador da eleição.
    pub const fn election_id(&self) -> ElectionId {
        self.election_id
    }

    /// Retorna o identificador do centro acadêmico.
    pub const fn academic_center_id(&self) -> AcademicCenterId {
        self.academic_center_id
    }

    /// Retorna as opções na ordem publicada.
    pub fn options(&self) -> &[ElectionOption] {
        self.options.as_slice()
    }

    /// Retorna as autoridades na ordem publicada.
    pub fn authorities(&self) -> &[Authority] {
        &self.authorities
    }

    /// Retorna a chave pública que o app usa para cifrar os votos desta eleição.
    pub const fn combined_public_key(&self) -> &U2048 {
        &self.combined_public_key
    }

    /// Retorna os bytes DER da chave de assinatura do servidor.
    pub fn server_signing_key(&self) -> &[u8] {
        self.server_signing_key.as_slice()
    }

    /// Retorna a versão do protocolo do manifesto.
    pub const fn version(&self) -> u8 {
        MANIFEST_VERSION
    }

    /// Retorna o identificador do grupo criptográfico.
    pub const fn group_id(&self) -> &'static str {
        ELECTION_GROUP_ID
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::modules::elections::domain::{
        entities::authority::KeyProof,
        identifiers::{AuthorityId, SlateId},
    };
    use uuid::Uuid;

    fn election_id() -> ElectionId {
        ElectionId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn academic_center_id() -> AcademicCenterId {
        AcademicCenterId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn authority_id() -> AuthorityId {
        AuthorityId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn slate_id() -> SlateId {
        SlateId::try_from_uuid(Uuid::new_v4()).unwrap()
    }

    fn slate_option(id: SlateId, name: &str) -> ElectionOption {
        ElectionOption::try_slate(id, name).unwrap()
    }

    fn null_option() -> ElectionOption {
        ElectionOption::Null
    }

    fn authority(role: AuthorityRole, id: AuthorityId, signing_key: &str) -> Authority {
        Authority::try_new(
            id,
            role,
            signing_key.as_bytes().to_vec(),
            U2048::from_u128(id.as_uuid().as_u128().max(2)),
            KeyProof::new(U2048::from_u32(456), U2048::from_u32(789)),
        )
        .unwrap()
    }

    fn server_authority(key: &str) -> Authority {
        authority(AuthorityRole::Server, authority_id(), key)
    }

    fn commission_authority() -> Authority {
        authority(AuthorityRole::Commission, authority_id(), "commission-key")
    }

    fn slate_authority(id: SlateId) -> Authority {
        authority(AuthorityRole::Slate(id), authority_id(), "slate-key")
    }

    fn fixture_slate_id() -> SlateId {
        SlateId::try_from_uuid(Uuid::from_u128(1)).unwrap()
    }

    fn valid_options() -> Vec<ElectionOption> {
        vec![
            slate_option(fixture_slate_id(), "Chapa Horizonte"),
            null_option(),
        ]
    }

    fn valid_authorities() -> Vec<Authority> {
        vec![
            server_authority("server-key"),
            commission_authority(),
            slate_authority(fixture_slate_id()),
        ]
    }

    fn valid_manifest_parts() -> (Vec<ElectionOption>, Vec<Authority>) {
        (valid_options(), valid_authorities())
    }

    fn build_manifest(
        options: Vec<ElectionOption>,
        authorities: Vec<Authority>,
        combined_public_key: u32,
        server_key: &str,
    ) -> Result<Manifest, InvalidManifest> {
        Manifest::try_new(
            election_id(),
            academic_center_id(),
            options,
            authorities,
            U2048::from_u32(combined_public_key),
            server_key.as_bytes().to_vec(),
        )
    }

    #[test]
    fn accepts_valid_manifest() {
        let manifest =
            build_manifest(valid_options(), valid_authorities(), 987_654, "server-key").unwrap();

        assert_eq!(manifest.version(), 1);
        assert_eq!(manifest.group_id(), "rfc3526-modp14");
        assert_eq!(manifest.options().len(), 2);
        assert_eq!(manifest.authorities().len(), 3);
    }

    #[test]
    fn accepts_manifest_with_maximum_options_and_authorities() {
        let mut options = Vec::with_capacity(MAX_OPTIONS);
        let mut authorities = vec![server_authority("server-key"), commission_authority()];

        for index in 1..=50 {
            let id = SlateId::try_from_uuid(Uuid::from_u128(index)).unwrap();

            options.push(slate_option(id, &format!("Chapa {index}")));
            authorities.push(authority(
                AuthorityRole::Slate(id),
                authority_id(),
                &format!("slate-key-{index}"),
            ));
        }

        options.push(null_option());

        let manifest = build_manifest(options, authorities, 987_654, "server-key").unwrap();

        assert_eq!(manifest.options().len(), MAX_OPTIONS);
        assert_eq!(manifest.authorities().len(), MAX_AUTHORITIES);

        assert_eq!(
            manifest
                .options()
                .iter()
                .filter(|option| matches!(option, ElectionOption::Slate(_)))
                .count(),
            50
        );

        assert_eq!(
            manifest
                .authorities()
                .iter()
                .filter(|authority| matches!(authority.role(), AuthorityRole::Slate(_)))
                .count(),
            50
        );
    }

    #[test]
    fn rejects_more_than_maximum_options() {
        let mut options = Vec::with_capacity(MAX_OPTIONS + 1);
        let mut authorities = vec![server_authority("server-key"), commission_authority()];

        for index in 1..=51 {
            let id = SlateId::try_from_uuid(Uuid::from_u128(index)).unwrap();

            options.push(slate_option(id, &format!("Chapa {index}")));
            authorities.push(slate_authority(id));
        }

        options.push(null_option());

        assert_eq!(
            build_manifest(options, authorities, 987_654, "server-key"),
            Err(InvalidManifest::InvalidOptionCount)
        );
    }

    #[test]
    fn rejects_more_than_maximum_authorities() {
        let mut authorities = vec![server_authority("server-key"), commission_authority()];

        for index in 1..=51 {
            let id = SlateId::try_from_uuid(Uuid::from_u128(index)).unwrap();

            authorities.push(slate_authority(id));
        }

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "server-key",),
            Err(InvalidManifest::InvalidAuthorityCount)
        );
    }

    #[test]
    fn preserves_option_and_authority_order() {
        let (options, authorities) = valid_manifest_parts();

        let manifest =
            build_manifest(options.clone(), authorities.clone(), 987_654, "server-key").unwrap();

        assert_eq!(manifest.options(), options.as_slice());
        assert_eq!(manifest.authorities(), authorities.as_slice());
    }

    #[test]
    fn rejects_too_few_options() {
        assert_eq!(
            build_manifest(
                vec![null_option()],
                valid_authorities(),
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::InvalidOptionCount)
        );
    }

    #[test]
    fn rejects_too_few_authorities() {
        assert_eq!(
            build_manifest(
                valid_options(),
                vec![server_authority("server-key"), commission_authority(),],
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::InvalidAuthorityCount)
        );
    }

    #[test]
    fn rejects_manifest_without_null_option() {
        let first_slate = slate_id();
        let second_slate = slate_id();

        assert_eq!(
            build_manifest(
                vec![
                    slate_option(first_slate, "Chapa A"),
                    slate_option(second_slate, "Chapa B"),
                ],
                vec![
                    server_authority("server-key"),
                    commission_authority(),
                    slate_authority(first_slate),
                    slate_authority(second_slate),
                ],
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::InvalidNullOptionCount)
        );
    }

    #[test]
    fn rejects_null_option_not_in_last_position() {
        let id = slate_id();

        assert_eq!(
            build_manifest(
                vec![null_option(), slate_option(id, "Chapa Horizonte"),],
                valid_authorities(),
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::NullOptionMustBeLast)
        );
    }

    #[test]
    fn rejects_duplicate_option_ids() {
        let id = slate_id();

        assert_eq!(
            build_manifest(
                vec![
                    slate_option(id, "Chapa A"),
                    slate_option(id, "Chapa B"),
                    null_option(),
                ],
                vec![
                    server_authority("server-key"),
                    commission_authority(),
                    slate_authority(id),
                ],
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::DuplicateOptionId)
        );
    }

    #[test]
    fn rejects_duplicate_authority_ids() {
        let shared_id = authority_id();

        let authorities = vec![
            authority(AuthorityRole::Server, shared_id, "server-key"),
            authority(AuthorityRole::Commission, shared_id, "commission-key"),
            slate_authority(slate_id()),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "server-key",),
            Err(InvalidManifest::DuplicateAuthorityId)
        );
    }

    #[test]
    fn rejects_duplicate_authority_signing_keys() {
        let authorities = vec![
            server_authority("shared-key"),
            authority(AuthorityRole::Commission, authority_id(), "shared-key"),
            slate_authority(slate_id()),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "shared-key",),
            Err(InvalidManifest::DuplicateAuthoritySigningKey)
        );
    }

    #[test]
    fn rejects_missing_server_authority() {
        let authorities = vec![
            commission_authority(),
            slate_authority(slate_id()),
            authority(
                AuthorityRole::Commission,
                authority_id(),
                "another-commission-key",
            ),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "server-key",),
            Err(InvalidManifest::InvalidServerAuthorityCount)
        );
    }

    #[test]
    fn rejects_missing_commission_authority() {
        let authorities = vec![
            server_authority("server-key"),
            slate_authority(slate_id()),
            authority(
                AuthorityRole::Slate(slate_id()),
                authority_id(),
                "another-slate-key",
            ),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "server-key",),
            Err(InvalidManifest::InvalidCommissionAuthorityCount)
        );
    }

    #[test]
    fn rejects_missing_slate_authority() {
        let first_slate = slate_id();
        let second_slate = slate_id();

        assert_eq!(
            build_manifest(
                vec![
                    slate_option(first_slate, "Chapa A"),
                    slate_option(second_slate, "Chapa B"),
                    null_option(),
                ],
                vec![
                    server_authority("server-key"),
                    commission_authority(),
                    slate_authority(first_slate),
                ],
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::SlateAuthorityMismatch)
        );
    }

    #[test]
    fn rejects_authority_for_unknown_slate() {
        let declared_slate = slate_id();
        let unknown_slate = slate_id();

        assert_eq!(
            build_manifest(
                vec![slate_option(declared_slate, "Chapa A"), null_option(),],
                vec![
                    server_authority("server-key"),
                    commission_authority(),
                    slate_authority(unknown_slate),
                ],
                987_654,
                "server-key",
            ),
            Err(InvalidManifest::SlateAuthorityMismatch)
        );
    }

    #[test]
    fn rejects_two_authorities_for_same_slate() {
        let id = slate_id();

        let authorities = vec![
            server_authority("server-key"),
            commission_authority(),
            slate_authority(id),
            authority(
                AuthorityRole::Slate(id),
                authority_id(),
                "another-slate-key",
            ),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "server-key",),
            Err(InvalidManifest::SlateAuthorityMismatch)
        );
    }

    #[test]
    fn accepts_server_signing_key_distinct_from_server_authority_key() {
        let manifest = build_manifest(
            valid_options(),
            valid_authorities(),
            987_654,
            "different-server-key",
        )
        .unwrap();

        assert_eq!(manifest.server_signing_key(), b"different-server-key");
        assert_eq!(manifest.authorities()[0].signing_key(), b"server-key");
    }

    #[test]
    fn rejects_duplicate_authority_public_values() {
        let mut authorities = valid_authorities();
        let commission = &authorities[1];

        authorities[1] = Authority::try_new(
            commission.id(),
            commission.role(),
            commission.signing_key().to_vec(),
            *authorities[0].public_value(),
            commission.key_proof().clone(),
        )
        .unwrap();

        assert_eq!(
            build_manifest(valid_options(), authorities, 987_654, "server-key"),
            Err(InvalidManifest::DuplicateAuthorityPublicValue)
        );
    }

    #[test]
    fn rejects_combined_public_keys_zero_and_one() {
        for public_key in [0, 1] {
            assert_eq!(
                build_manifest(
                    valid_options(),
                    valid_authorities(),
                    public_key,
                    "server-key"
                ),
                Err(InvalidManifest::InvalidCombinedPublicKey)
            );
        }
    }
}
