//! Conversões de representação JSON da API eleitoral.
//!
//! Este módulo conhece as codificações do transporte (UUID textual canônico,
//! hexadecimal, Base64url e inteiro decimal), mas não decide elegibilidade,
//! assinatura válida, manifestação de voto ou regras do ElGamal.

use ::serde::{Deserialize, Deserializer, Serializer, de::Error as _};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crypto_bigint::U2048;
use uuid::Uuid;

/// Lê um UUID canônico, permitindo o valor nulo reservado para a opção Nulo.
pub(super) fn option_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    parse_uuid::<D::Error>(&text)
}

/// Lê um UUID canônico não nulo.
pub(super) fn non_nil_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let value = option_uuid(deserializer)?;
    if value.is_nil() {
        return Err(D::Error::custom("UUID não pode ser nulo"));
    }
    Ok(value)
}

/// O campo deve estar presente, mas pode conter null.
/// `deserialize_with` evita tratar campo ausente como null implicitamente.
pub(super) fn nullable_non_nil_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = Option::<String>::deserialize(deserializer)?;
    text.map(|text| {
        let value = parse_uuid::<D::Error>(&text)?;
        if value.is_nil() {
            return Err(D::Error::custom("UUID não pode ser nulo"));
        }
        Ok(value)
    })
    .transpose()
}

/// Aceita exclusivamente a versão inteira 1 do formato público atual.
/// O valor não é transportado para a aplicação.
pub(super) fn version_one<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    if u8::deserialize(deserializer)? != 1 {
        return Err(D::Error::custom("versão eleitoral não suportada"));
    }
    Ok(())
}

fn parse_uuid<E: ::serde::de::Error>(text: &str) -> Result<Uuid, E> {
    let value = Uuid::parse_str(text).map_err(E::custom)?;
    if value.to_string() != text {
        return Err(E::custom("UUID deve usar a forma canônica em minúsculas"));
    }
    Ok(value)
}

/// Lê o digest SHA-256 como 64 caracteres hexadecimais minúsculos.
pub(super) fn sha256_hex<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    let raw = text.as_bytes();
    if raw.len() != 64 {
        return Err(D::Error::custom("SHA-256 deve ter 64 dígitos hexadecimais"));
    }

    let mut result = [0_u8; 32];
    for (index, byte) in result.iter_mut().enumerate() {
        let hi = hex_nibble(raw[index * 2])
            .ok_or_else(|| D::Error::custom("SHA-256 deve conter somente hexadecimal minúsculo"))?;
        let lo = hex_nibble(raw[index * 2 + 1])
            .ok_or_else(|| D::Error::custom("SHA-256 deve conter somente hexadecimal minúsculo"))?;
        *byte = (hi << 4) | lo;
    }
    Ok(result)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

/// Lê os bytes de uma chave SPKI DER ou assinatura codificada em Base64url
/// canônico, sem padding. O limite é o do contrato eleitoral atual.
pub(super) fn base64url<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    if text.is_empty() || text.len() > 1024 {
        return Err(D::Error::custom("Base64url fora do limite permitido"));
    }

    let bytes = URL_SAFE_NO_PAD.decode(&text).map_err(D::Error::custom)?;
    if URL_SAFE_NO_PAD.encode(&bytes) != text {
        return Err(D::Error::custom("Base64url não canônico"));
    }
    Ok(bytes)
}

/// Lê inteiros de até 2048 bits em representação decimal sem zeros à esquerda.
/// Valores fora do subgrupo ElGamal continuam sendo responsabilidade do domínio.
pub(super) fn decimal_u2048<'de, D>(deserializer: D) -> Result<U2048, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    if text.is_empty()
        || text.len() > 617
        || !text.bytes().all(|byte| byte.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return Err(D::Error::custom("inteiro decimal não canônico"));
    }
    U2048::from_str_radix_vartime(&text, 10)
        .map_err(|_| D::Error::custom("inteiro decimal excede 2048 bits"))
}

/// Codifica a chave opcional da resposta de cadastro como Base64url sem padding.
pub(super) fn optional_base64url<S>(
    bytes: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match bytes {
        Some(value) => serializer.serialize_some(&URL_SAFE_NO_PAD.encode(value)),
        None => serializer.serialize_none(),
    }
}
