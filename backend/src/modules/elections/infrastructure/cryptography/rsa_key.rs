//! Perfil RSA/SPKI comum aos verificadores de cédulas e cadastro.

use rsa::{
    BigUint, RsaPublicKey,
    pkcs8::{DecodePublicKey, EncodePublicKey},
    traits::PublicKeyParts,
};

/// Exige DER/SPKI canônico, RSA 2048/3072/4096 e expoente 65537.
pub(super) fn decode_rsa_public_key(der: &[u8]) -> Result<RsaPublicKey, ()> {
    let key = RsaPublicKey::from_public_key_der(der).map_err(|_| ())?;
    if !matches!(key.n().bits(), 2048 | 3072 | 4096)
        || key.e() != &BigUint::from(65_537_u32)
        || key.to_public_key_der().map_err(|_| ())?.as_bytes() != der
    {
        return Err(());
    }
    Ok(key)
}
