//! Manifesto imutável que define a configuração de uma eleição.

use std::collections::HashSet;

use crate::modules::elections::domain::{
    entities::authority::{Authority, AuthorityRole},
    value_objects::{
        base64_url::Base64Url,
        big_uint::BigUInt,
        fingerprint::Fingerprint,
        identifiers::{AcademicCenterId, ElectionId},
        option::{ElectionOption, OptionKind},
        options::{ElectionOptions, InvalidElectionOptions},
    },
};

/// Identificador do grupo criptográfico definido pelo protocolo v1.
pub const ELECTION_GROUP_ID: &str = "rfc3526-modp14";

// Preserva o caminho público usado pelos consumidores do manifesto.
pub use crate::modules::elections::domain::value_objects::options::{MAX_OPTIONS, MIN_OPTIONS};

/// Quantidade mínima de autoridades: servidor, comissão e uma chapa.
pub const MIN_AUTHORITIES: usize = 3;

/// Quantidade máxima de autoridades: servidor, comissão e 50 chapas.
pub const MAX_AUTHORITIES: usize = 52;

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
/// use backend::modules::elections::domain::{entities::manifest::{Manifest, VerifiedManifest},
///     value_objects::fingerprint::Fingerprint};
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
/// da configuração publicada pelo protocolo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    election_id: ElectionId,
    academic_center_id: AcademicCenterId,
    options: ElectionOptions,
    authorities: Vec<Authority>,
    combined_public_key: BigUInt,
    server_signing_key: Base64Url,
    calendar_origin_hash: Fingerprint,
}

/// Erros encontrados ao construir um manifesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidManifest {
    /// Quantidade de opções fora dos limites do protocolo.
    #[error("O manifesto deve possuir entre 2 e 51 opções")]
    InvalidOptionCount,

    /// Quantidade de autoridades fora dos limites do protocolo.
    #[error("O manifesto deve possuir entre 3 e 52 autoridades")]
    InvalidAuthorityCount,

    /// O manifesto precisa conter exatamente uma opção Nulo.
    #[error("O manifesto deve possuir exatamente uma opção Nulo")]
    InvalidNullOptionCount,

    /// A opção Nulo deve ser a última opção.
    #[error("A opção Nulo deve ocupar a última posição")]
    NullOptionMustBeLast,

    /// Duas opções não podem compartilhar o mesmo identificador.
    #[error("Os identificadores das opções devem ser distintos")]
    DuplicateOptionId,

    /// Duas autoridades não podem compartilhar o mesmo identificador.
    #[error("Os identificadores das autoridades devem ser distintos")]
    DuplicateAuthorityId,

    /// Duas autoridades não podem compartilhar a mesma chave de assinatura.
    #[error("As chaves de assinatura das autoridades devem ser distintas")]
    DuplicateAuthoritySigningKey,

    /// As contribuições públicas das autoridades devem ser distintas.
    #[error("Os valores públicos das autoridades devem ser distintos")]
    DuplicateAuthorityPublicValue,

    /// Deve existir exatamente uma autoridade do servidor.
    #[error("O manifesto deve possuir exatamente uma autoridade Server")]
    InvalidServerAuthorityCount,

    /// Deve existir exatamente uma autoridade da comissão.
    #[error("O manifesto deve possuir exatamente uma autoridade da comissão")]
    InvalidCommissionAuthorityCount,

    /// Deve existir uma autoridade para cada chapa.
    #[error("Deve existir exatamente uma autoridade para cada chapa")]
    SlateAuthorityMismatch,

    /// A chave pública combinada não pode ser zero ou um.
    #[error("A chave pública combinada não pode ser zero ou um")]
    InvalidCombinedPublicKey,
}

impl Manifest {
    /// Constrói um manifesto validando suas invariantes estruturais.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        election_id: ElectionId,
        academic_center_id: AcademicCenterId,
        options: Vec<ElectionOption>,
        authorities: Vec<Authority>,
        combined_public_key: BigUInt,
        server_signing_key: Base64Url,
        calendar_origin_hash: Fingerprint,
    ) -> Result<Self, InvalidManifest> {
        let options = ElectionOptions::try_new(options).map_err(|error| match error {
            InvalidElectionOptions::InvalidCount => InvalidManifest::InvalidOptionCount,
            InvalidElectionOptions::DuplicateId => InvalidManifest::DuplicateOptionId,
            InvalidElectionOptions::InvalidNullCount => InvalidManifest::InvalidNullOptionCount,
        })?;
        if options.as_slice().last().map(ElectionOption::kind) != Some(OptionKind::Null) {
            return Err(InvalidManifest::NullOptionMustBeLast);
        }

        if !(MIN_AUTHORITIES..=MAX_AUTHORITIES).contains(&authorities.len()) {
            return Err(InvalidManifest::InvalidAuthorityCount);
        }

        if matches!(combined_public_key.as_str(), "0" | "1") {
            return Err(InvalidManifest::InvalidCombinedPublicKey);
        }

        let slate_ids: HashSet<_> = options
            .as_slice()
            .iter()
            .filter(|option| option.kind() == OptionKind::Slate)
            .map(|option| *option.id().as_uuid())
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

            if !signing_keys.insert(authority.signing_key().as_str()) {
                return Err(InvalidManifest::DuplicateAuthoritySigningKey);
            }
            if !public_values.insert(authority.public_value().as_str()) {
                return Err(InvalidManifest::DuplicateAuthorityPublicValue);
            }

            match authority.role() {
                AuthorityRole::Server => {
                    server_authority_count += 1;
                }
                AuthorityRole::Commission => {
                    commission_authority_count += 1;
                }
                AuthorityRole::Slate => {
                    let slate_id = authority
                        .slate_id()
                        .expect("Authority::try_new garante slate_id para Slate");

                    if !authority_slate_ids.insert(*slate_id.as_uuid()) {
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
            calendar_origin_hash,
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

    /// Retorna a chave pública combinada.
    pub const fn combined_public_key(&self) -> &BigUInt {
        &self.combined_public_key
    }

    /// Retorna a chave de assinatura do servidor.
    pub fn server_signing_key(&self) -> &Base64Url {
        &self.server_signing_key
    }

    /// Retorna o hash da origem do calendário.
    pub const fn calendar_origin_hash(&self) -> &Fingerprint {
        &self.calendar_origin_hash
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
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    fn encoded_key(value: &str) -> String {
        URL_SAFE_NO_PAD.encode(value.as_bytes())
    }

    use super::*;
    use crate::modules::elections::domain::value_objects::{
        identifiers::{AuthorityId, OptionId, SlateId},
        key_proof::KeyProof,
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
        ElectionOption::try_new(OptionId::from_uuid(id.into_uuid()), OptionKind::Slate, name)
            .unwrap()
    }

    fn null_option() -> ElectionOption {
        ElectionOption::try_new(
            OptionId::from_uuid(Uuid::nil()),
            OptionKind::Null,
            ElectionOption::NULL_OPTION_NAME,
        )
        .unwrap()
    }

    fn authority(
        role: AuthorityRole,
        slate_id: Option<SlateId>,
        id: AuthorityId,
        signing_key: &str,
    ) -> Authority {
        Authority::try_new(
            id,
            role,
            slate_id,
            Base64Url::try_new(encoded_key(signing_key)).unwrap(),
            BigUInt::try_from_decimal(id.as_uuid().as_u128().max(2).to_string()).unwrap(),
            KeyProof::new(
                BigUInt::try_from_decimal("456").unwrap(),
                BigUInt::try_from_decimal("789").unwrap(),
            ),
        )
        .unwrap()
    }

    fn server_authority(key: &str) -> Authority {
        authority(AuthorityRole::Server, None, authority_id(), key)
    }

    fn commission_authority() -> Authority {
        authority(
            AuthorityRole::Commission,
            None,
            authority_id(),
            "commission-key",
        )
    }

    fn slate_authority(id: SlateId) -> Authority {
        authority(AuthorityRole::Slate, Some(id), authority_id(), "slate-key")
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
        combined_public_key: &str,
        server_key: &str,
    ) -> Result<Manifest, InvalidManifest> {
        Manifest::try_new(
            election_id(),
            academic_center_id(),
            options,
            authorities,
            BigUInt::try_from_decimal(combined_public_key).unwrap(),
            Base64Url::try_new(encoded_key(server_key)).unwrap(),
            Fingerprint::try_from_hex("a".repeat(64)).unwrap(),
        )
    }

    #[test]
    fn accepts_valid_manifest() {
        let manifest =
            build_manifest(valid_options(), valid_authorities(), "987654", "server-key").unwrap();

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
                AuthorityRole::Slate,
                Some(id),
                authority_id(),
                &format!("slate-key-{index}"),
            ));
        }

        options.push(null_option());

        let manifest = build_manifest(options, authorities, "987654", "server-key").unwrap();

        assert_eq!(manifest.options().len(), MAX_OPTIONS);
        assert_eq!(manifest.authorities().len(), MAX_AUTHORITIES);

        assert_eq!(
            manifest
                .options()
                .iter()
                .filter(|option| option.kind() == OptionKind::Slate)
                .count(),
            50
        );

        assert_eq!(
            manifest
                .authorities()
                .iter()
                .filter(|authority| authority.role() == AuthorityRole::Slate)
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
            build_manifest(options, authorities, "987654", "server-key"),
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
            build_manifest(valid_options(), authorities, "987654", "server-key",),
            Err(InvalidManifest::InvalidAuthorityCount)
        );
    }

    #[test]
    fn preserves_option_and_authority_order() {
        let (options, authorities) = valid_manifest_parts();

        let manifest =
            build_manifest(options.clone(), authorities.clone(), "987654", "server-key").unwrap();

        assert_eq!(manifest.options(), options.as_slice());
        assert_eq!(manifest.authorities(), authorities.as_slice());
    }

    #[test]
    fn rejects_too_few_options() {
        assert_eq!(
            build_manifest(
                vec![null_option()],
                valid_authorities(),
                "987654",
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
                "987654",
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
                "987654",
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
                "987654",
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
                "987654",
                "server-key",
            ),
            Err(InvalidManifest::DuplicateOptionId)
        );
    }

    #[test]
    fn rejects_duplicate_authority_ids() {
        let shared_id = authority_id();

        let authorities = vec![
            authority(AuthorityRole::Server, None, shared_id, "server-key"),
            authority(AuthorityRole::Commission, None, shared_id, "commission-key"),
            slate_authority(slate_id()),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, "987654", "server-key",),
            Err(InvalidManifest::DuplicateAuthorityId)
        );
    }

    #[test]
    fn rejects_duplicate_authority_signing_keys() {
        let authorities = vec![
            server_authority("shared-key"),
            authority(
                AuthorityRole::Commission,
                None,
                authority_id(),
                "shared-key",
            ),
            slate_authority(slate_id()),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, "987654", "shared-key",),
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
                None,
                authority_id(),
                "another-commission-key",
            ),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, "987654", "server-key",),
            Err(InvalidManifest::InvalidServerAuthorityCount)
        );
    }

    #[test]
    fn rejects_missing_commission_authority() {
        let authorities = vec![
            server_authority("server-key"),
            slate_authority(slate_id()),
            authority(
                AuthorityRole::Slate,
                Some(slate_id()),
                authority_id(),
                "another-slate-key",
            ),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, "987654", "server-key",),
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
                "987654",
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
                "987654",
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
                AuthorityRole::Slate,
                Some(id),
                authority_id(),
                "another-slate-key",
            ),
        ];

        assert_eq!(
            build_manifest(valid_options(), authorities, "987654", "server-key",),
            Err(InvalidManifest::SlateAuthorityMismatch)
        );
    }

    #[test]
    fn accepts_server_signing_key_distinct_from_server_authority_key() {
        let manifest = build_manifest(
            valid_options(),
            valid_authorities(),
            "987654",
            "different-server-key",
        )
        .unwrap();

        assert_eq!(
            manifest.server_signing_key().as_str(),
            encoded_key("different-server-key")
        );
        assert_eq!(
            manifest.authorities()[0].signing_key().as_str(),
            encoded_key("server-key")
        );
    }

    #[test]
    fn rejects_combined_public_key_equal_to_one() {
        assert_eq!(
            build_manifest(valid_options(), valid_authorities(), "1", "server-key",),
            Err(InvalidManifest::InvalidCombinedPublicKey)
        );
    }
}
