//! Codificações canônicas compartilhadas pelos codecs de artefatos.

use rsa::BigUint;

pub(super) fn decimal(value: &crypto_bigint::U2048) -> String {
    BigUint::from_bytes_be(value.to_be_bytes().as_slice()).to_str_radix(10)
}

pub(super) fn lower_hex(bytes: &[u8; 32]) -> String {
    const CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(64);
    for &byte in bytes {
        result.push(char::from(CHARS[usize::from(byte >> 4)]));
        result.push(char::from(CHARS[usize::from(byte & 0x0f)]));
    }
    result
}
