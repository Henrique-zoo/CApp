//! Codificações dos campos binários no JSON eleitoral v1.
//!
//! Os adaptadores leem Base64url e hexadecimal canônicos antes de construir
//! os tipos internos. A escrita produz o mesmo formato para o protocolo assinado.

use std::mem::size_of;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::modules::elections::domain::Fingerprint;

/// Comprimento máximo de um campo Base64url no contrato eleitoral.
pub const BASE64_URL_MAX_LENGTH: usize = 1024;

const FINGERPRINT_HEX_LENGTH: usize = size_of::<Fingerprint>() * 2;

/// Falha ao converter um campo Base64url.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidBase64Url {
    /// O contrato exige um campo não vazio.
    #[error("Base64url não pode ser vazio")]
    Empty,

    /// A forma textual excede o limite do contrato.
    #[error("Base64url excede 1024 caracteres")]
    TooLong,

    /// A entrada precisa ser Base64url canônico sem padding.
    #[error("Base64url deve usar a forma canônica sem padding")]
    InvalidEncoding,
}

/// Digest fora do formato hexadecimal aceito pelo contrato eleitoral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("O hash deve conter exatamente 64 caracteres hexadecimais minúsculos")]
pub struct InvalidFingerprint;

/// Decodifica um campo Base64url sem padding, com o limite do contrato.
///
/// Retorna erro para campos vazios, longos ou fora da forma canônica.
pub fn decode_base64_url(value: &str) -> Result<Vec<u8>, InvalidBase64Url> {
    validate_base64_url_length(value.len())?;

    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| InvalidBase64Url::InvalidEncoding)
}

/// Produz a forma Base64url de uma chave pública ou assinatura para o JSON.
///
/// Retorna erro se o valor for vazio ou se a forma textual exceder o limite.
pub fn encode_base64_url(bytes: &[u8]) -> Result<String, InvalidBase64Url> {
    let length = base64::encoded_len(bytes.len(), false).ok_or(InvalidBase64Url::TooLong)?;
    validate_base64_url_length(length)?;

    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn validate_base64_url_length(length: usize) -> Result<(), InvalidBase64Url> {
    match length {
        0 => Err(InvalidBase64Url::Empty),
        1..=BASE64_URL_MAX_LENGTH => Ok(()),
        _ => Err(InvalidBase64Url::TooLong),
    }
}

/// Lê os 64 dígitos hexadecimais minúsculos de um digest do contrato.
pub fn decode_fingerprint(hex: &str) -> Result<Fingerprint, InvalidFingerprint> {
    if hex.len() != FINGERPRINT_HEX_LENGTH {
        return Err(InvalidFingerprint);
    }

    let digit = |byte| match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(InvalidFingerprint),
    };

    let (pairs, _) = hex.as_bytes().as_chunks::<2>();
    let mut digest = [0; size_of::<Fingerprint>()];
    for (&[high, low], byte) in pairs.iter().zip(&mut digest) {
        *byte = digit(high)? << 4 | digit(low)?;
    }

    Ok(digest)
}

/// Escreve um digest como 64 dígitos hexadecimais minúsculos para o JSON.
pub fn encode_fingerprint(digest: &Fingerprint) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(FINGERPRINT_HEX_LENGTH);

    for &byte in digest {
        hex.push(char::from(HEX[usize::from(byte >> 4)]));
        hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }

    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_url_round_trips_without_padding() {
        for (bytes, encoded) in [(b"a".as_slice(), "YQ"), (&[0xfb, 0xff, 0xff], "-___")] {
            assert_eq!(decode_base64_url(encoded).unwrap(), bytes);
            assert_eq!(encode_base64_url(bytes).unwrap(), encoded);
        }
    }

    #[test]
    fn base64_url_rejects_noncanonical_text() {
        for encoded in ["abc=", "abc+/", "YR", "Zh", "Zm9"] {
            assert_eq!(
                decode_base64_url(encoded),
                Err(InvalidBase64Url::InvalidEncoding)
            );
        }
    }

    #[test]
    fn base64_url_enforces_the_protocol_limits_in_both_directions() {
        assert_eq!(decode_base64_url(""), Err(InvalidBase64Url::Empty));
        assert_eq!(encode_base64_url(&[]), Err(InvalidBase64Url::Empty));

        let largest = vec![0; 768];
        let encoded = encode_base64_url(&largest).unwrap();
        assert_eq!(encoded.len(), BASE64_URL_MAX_LENGTH);
        assert_eq!(decode_base64_url(&encoded).unwrap(), largest);

        assert_eq!(
            decode_base64_url(&"a".repeat(BASE64_URL_MAX_LENGTH + 1)),
            Err(InvalidBase64Url::TooLong)
        );
        assert_eq!(encode_base64_url(&[0; 769]), Err(InvalidBase64Url::TooLong));
    }

    #[test]
    fn fingerprint_round_trips_in_lowercase_hex() {
        let encoded = "0123456789abcdef".repeat(4);
        let digest = decode_fingerprint(&encoded).unwrap();

        assert_eq!(
            digest[..8],
            [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
        );
        assert_eq!(encode_fingerprint(&digest), encoded);
    }

    #[test]
    fn fingerprint_rejects_invalid_lengths_and_characters() {
        for value in [
            String::new(),
            "a".repeat(63),
            "a".repeat(65),
            "A".repeat(64),
            format!("{}g", "a".repeat(63)),
        ] {
            assert_eq!(decode_fingerprint(&value), Err(InvalidFingerprint));
        }
    }
}
