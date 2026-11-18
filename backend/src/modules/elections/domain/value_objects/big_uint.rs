//! Inteiro decimal sem sinal representado sem perda de precisão.

/// Comprimento máximo permitido pelo contrato eleitoral.
pub const BIG_UINT_MAX_LENGTH: usize = 617;

/// Inteiro decimal sem sinal em representação canônica.
///
/// O valor é armazenado como `String` para preservar números maiores que os
/// tipos inteiros nativos. A representação não possui sinal nem zeros à esquerda,
/// exceto pelo próprio valor zero.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigUInt(String);

/// Motivo pelo qual uma representação decimal não foi aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBigUInt {
    /// A representação está vazia.
    #[error("BigUInt não pode ser vazio")]
    Empty,

    /// A representação contém caracteres diferentes de dígitos ASCII.
    #[error("BigUInt deve conter somente dígitos decimais ASCII")]
    NonDecimal,

    /// A representação possui zeros à esquerda.
    #[error("BigUInt não pode conter zeros à esquerda")]
    LeadingZero,

    /// A representação excede o limite do contrato.
    #[error("BigUInt excede o limite de 617 caracteres")]
    TooLong,
}

impl BigUInt {
    /// Constrói um inteiro decimal sem sinal validando sua representação.
    pub fn try_from_decimal(value: impl Into<String>) -> Result<Self, InvalidBigUInt> {
        let value = value.into();

        if value.is_empty() {
            return Err(InvalidBigUInt::Empty);
        }

        if value.len() > BIG_UINT_MAX_LENGTH {
            return Err(InvalidBigUInt::TooLong);
        }

        if !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(InvalidBigUInt::NonDecimal);
        }

        if value.len() > 1 && value.starts_with('0') {
            return Err(InvalidBigUInt::LeadingZero);
        }

        Ok(Self(value))
    }

    /// Retorna a representação decimal sem transferir a propriedade.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consome o objeto e retorna sua representação decimal.
    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero() {
        let value = BigUInt::try_from_decimal("0").unwrap();

        assert_eq!(value.as_str(), "0");
    }

    #[test]
    fn accepts_positive_decimal_integer() {
        let value = BigUInt::try_from_decimal("123456789").unwrap();

        assert_eq!(value.as_str(), "123456789");
    }

    #[test]
    fn accepts_maximum_length() {
        let decimal = format!("1{}", "0".repeat(BIG_UINT_MAX_LENGTH - 1));
        let value = BigUInt::try_from_decimal(decimal.clone()).unwrap();

        assert_eq!(value.as_str(), decimal);
    }

    #[test]
    fn rejects_empty_value() {
        assert_eq!(BigUInt::try_from_decimal(""), Err(InvalidBigUInt::Empty));
    }

    #[test]
    fn rejects_non_decimal_characters() {
        assert_eq!(
            BigUInt::try_from_decimal("12a3"),
            Err(InvalidBigUInt::NonDecimal)
        );

        assert_eq!(
            BigUInt::try_from_decimal("-123"),
            Err(InvalidBigUInt::NonDecimal)
        );

        assert_eq!(
            BigUInt::try_from_decimal("+123"),
            Err(InvalidBigUInt::NonDecimal)
        );
    }

    #[test]
    fn rejects_leading_zeroes() {
        assert_eq!(
            BigUInt::try_from_decimal("00"),
            Err(InvalidBigUInt::LeadingZero)
        );

        assert_eq!(
            BigUInt::try_from_decimal("0123"),
            Err(InvalidBigUInt::LeadingZero)
        );
    }

    #[test]
    fn rejects_value_longer_than_contract_limit() {
        let decimal = format!("1{}", "0".repeat(BIG_UINT_MAX_LENGTH));

        assert_eq!(
            BigUInt::try_from_decimal(decimal),
            Err(InvalidBigUInt::TooLong)
        );
    }

    #[test]
    fn preserves_large_integer_without_numeric_conversion() {
        let decimal = format!("9{}", "8".repeat(BIG_UINT_MAX_LENGTH - 1));
        let value = BigUInt::try_from_decimal(decimal.clone()).unwrap();

        assert_eq!(value.into_string(), decimal);
    }
}
