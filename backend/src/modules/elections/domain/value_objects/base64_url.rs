//! Representação Base64url canônica sem padding.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// Comprimento máximo permitido pelo contrato eleitoral.
pub const BASE64_URL_MAX_LENGTH: usize = 1024;

/// Texto codificado em Base64url sem padding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Base64Url(String);

/// Erro de representação Base64url inválida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBase64Url {
    /// O valor não pode ser vazio.
    #[error("Base64url não pode ser vazio")]
    Empty,

    /// O valor excede o comprimento permitido.
    #[error("Base64url excede 1024 caracteres")]
    TooLong,

    /// O valor contém caracteres inválidos ou padding.
    #[error("Base64url contém caracteres inválidos ou padding")]
    InvalidCharacter,

    /// A representação não é canônica.
    #[error("Base64url não possui representação canônica")]
    NonCanonical,
}

impl Base64Url {
    /// Constrói uma representação Base64url canônica sem padding.
    pub fn try_new(value: impl Into<String>) -> Result<Self, InvalidBase64Url> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidBase64Url::Empty);
        }

        if value.len() > BASE64_URL_MAX_LENGTH {
            return Err(InvalidBase64Url::TooLong);
        }

        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Err(InvalidBase64Url::InvalidCharacter);
        }

        let decoded = URL_SAFE_NO_PAD
            .decode(value.as_bytes())
            .map_err(|_| InvalidBase64Url::InvalidCharacter)?;

        let encoded = URL_SAFE_NO_PAD.encode(decoded);

        if encoded != value {
            return Err(InvalidBase64Url::NonCanonical);
        }

        Ok(Self(value))
    }

    /// Retorna a representação sem transferir a propriedade.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consome o objeto e retorna a representação.
    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_base64url() {
        let value = Base64Url::try_new("YQ").unwrap();

        assert_eq!(value.as_str(), "YQ");
    }

    #[test]
    fn accepts_url_safe_special_characters() {
        // Os bytes [0xfb, 0xff, 0xff] geram "-___" em Base64url.
        let encoded = URL_SAFE_NO_PAD.encode([0xfb, 0xff, 0xff]);

        assert_eq!(encoded, "-___");

        let value = Base64Url::try_new(encoded).unwrap();

        assert_eq!(value.as_str(), "-___");
    }

    #[test]
    fn rejects_empty_value() {
        assert_eq!(Base64Url::try_new(""), Err(InvalidBase64Url::Empty));
    }

    #[test]
    fn rejects_padding() {
        assert_eq!(
            Base64Url::try_new("abc="),
            Err(InvalidBase64Url::InvalidCharacter)
        );
    }

    #[test]
    fn rejects_invalid_characters() {
        assert_eq!(
            Base64Url::try_new("abc+/"),
            Err(InvalidBase64Url::InvalidCharacter)
        );
    }

    #[test]
    fn rejects_non_canonical_trailing_bits() {
        for value in ["YR", "Zh", "Zm9"] {
            assert!(
                Base64Url::try_new(value).is_err(),
                "A representação {value} deveria ser rejeitada"
            );
        }
    }

    #[test]
    fn accepts_maximum_length() {
        // 1024 caracteres é múltiplo de quatro e forma uma codificação válida.
        let value = URL_SAFE_NO_PAD.encode(vec![0_u8; 768]);

        assert_eq!(value.len(), BASE64_URL_MAX_LENGTH);
        assert!(Base64Url::try_new(value).is_ok());
    }

    #[test]
    fn rejects_value_longer_than_limit() {
        let value = "a".repeat(BASE64_URL_MAX_LENGTH + 1);

        assert_eq!(Base64Url::try_new(value), Err(InvalidBase64Url::TooLong));
    }
}
