//! Hash SHA-256 hexadecimal usado como impressão digital no protocolo.

/// Quantidade de caracteres de um hash SHA-256 hexadecimal.
pub const FINGERPRINT_LENGTH: usize = 64;

/// Hash hexadecimal SHA-256 em formato canônico.
///
/// O valor contém exatamente 64 caracteres hexadecimais minúsculos.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint(String);

/// Erro ao construir uma impressão digital inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("Fingerprint deve conter exatamente 64 caracteres hexadecimais minúsculos")]
pub struct InvalidFingerprint;

impl Fingerprint {
    /// Constrói uma impressão digital a partir de sua representação hexadecimal.
    pub fn try_from_hex(value: impl Into<String>) -> Result<Self, InvalidFingerprint> {
        let value = value.into();

        if value.len() != FINGERPRINT_LENGTH
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(InvalidFingerprint);
        }

        Ok(Self(value))
    }

    /// Retorna a representação hexadecimal sem transferir a propriedade.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consome o objeto e retorna sua representação hexadecimal.
    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_lowercase_sha256_fingerprint() {
        let value = "a".repeat(FINGERPRINT_LENGTH);
        let fingerprint = Fingerprint::try_from_hex(value.clone()).unwrap();

        assert_eq!(fingerprint.as_str(), value);
    }

    #[test]
    fn rejects_fingerprint_with_wrong_length() {
        assert!(Fingerprint::try_from_hex("a".repeat(63)).is_err());
        assert!(Fingerprint::try_from_hex("a".repeat(65)).is_err());
    }

    #[test]
    fn rejects_uppercase_hexadecimal_characters() {
        let value = "A".repeat(FINGERPRINT_LENGTH);

        assert!(Fingerprint::try_from_hex(value).is_err());
    }

    #[test]
    fn rejects_non_hexadecimal_characters() {
        let value = format!("{}g", "a".repeat(63));

        assert!(Fingerprint::try_from_hex(value).is_err());
    }

    #[test]
    fn rejects_empty_fingerprint() {
        assert!(Fingerprint::try_from_hex("").is_err());
    }

    #[test]
    fn preserves_the_original_representation() {
        let value = "0123456789abcdef".repeat(4);
        let fingerprint = Fingerprint::try_from_hex(value.clone()).unwrap();

        assert_eq!(fingerprint.into_string(), value);
    }
}
