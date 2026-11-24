//! Configura a terminação TLS da API antes da abertura do socket.
//!
//! O executável usa o certificado e a chave PEM para atender HTTPS com Rustls.
//! O material TLS fica fora do router e do estado da aplicação.

use std::{env, path::Path, sync::Arc};

use anyhow::{Context, ensure};
use axum_server::tls_rustls::RustlsConfig;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    version::{TLS12, TLS13},
};
use x509_parser::{certificate::X509Certificate, parse_x509_certificate};

fn parse_certificate(der: &[u8]) -> anyhow::Result<X509Certificate<'_>> {
    let (trailing, certificate) = parse_x509_certificate(der)
        .map_err(|err| anyhow::anyhow!("Certificado X.509 inválido: {err}"))?;
    ensure!(trailing.is_empty(), "Dados após o certificado DER");
    ensure!(
        certificate.validity().is_valid(),
        "Certificado fora do período de validade"
    );
    Ok(certificate)
}

/// Carrega a configuração do servidor e retorna também o SPKI DER do certificado.
///
/// `BACKEND_TLS_CERT_FILE` aponta para o PEM do servidor com eventuais
/// intermediários em ordem. `BACKEND_TLS_KEY_FILE` aponta para a chave privada
/// correspondente. O ponto de composição usa o SPKI para verificar que a chave
/// TLS difere da chave de assinatura eleitoral.
///
/// # Erros
///
/// Falha quando os caminhos não estão definidos, os arquivos não podem ser
/// carregados, o certificado está fora do período de validade ou sem SAN, ou
/// a chave privada é inválida ou incompatível com o certificado.
pub fn load_from_env() -> anyhow::Result<(RustlsConfig, Vec<u8>)> {
    let cert_path = env::var_os("BACKEND_TLS_CERT_FILE")
        .filter(|path| !path.is_empty())
        .context("Defina BACKEND_TLS_CERT_FILE com o caminho do certificado PEM")?;
    let key_path = env::var_os("BACKEND_TLS_KEY_FILE")
        .filter(|path| !path.is_empty())
        .context("Defina BACKEND_TLS_KEY_FILE com o caminho da chave privada PEM")?;

    let cert_path = Path::new(&cert_path);
    let key_path = Path::new(&key_path);
    let certificate_error = || {
        format!(
            "Certificado em BACKEND_TLS_CERT_FILE ({})",
            cert_path.display()
        )
    };

    let chain = CertificateDer::pem_file_iter(cert_path)
        .with_context(certificate_error)?
        .collect::<Result<Vec<_>, _>>()
        .with_context(certificate_error)?;
    let (leaf, intermediates) = chain
        .split_first()
        .with_context(|| format!("{} não contém um certificado PEM", certificate_error()))?;

    let leaf = parse_certificate(leaf).with_context(certificate_error)?;
    let san = leaf
        .subject_alternative_name()
        .with_context(certificate_error)?;
    ensure!(
        san.is_some(),
        "{} exige um Subject Alternative Name (SAN)",
        certificate_error()
    );
    let leaf_spki = leaf.public_key().raw.to_vec();
    for intermediate in intermediates {
        parse_certificate(intermediate).with_context(certificate_error)?;
    }

    let key = PrivateKeyDer::from_pem_file(key_path).with_context(|| {
        format!(
            "Chave privada PEM em BACKEND_TLS_KEY_FILE ({})",
            key_path.display()
        )
    })?;

    let mut config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_protocol_versions(&[&TLS13, &TLS12])
            .context("Não foi possível configurar os protocolos TLS do backend")?
            .with_no_client_auth()
            .with_single_cert(chain, key)
            .with_context(|| {
                format!(
                    "Certificado TLS inválido ou incompatível com BACKEND_TLS_KEY_FILE ({})",
                    key_path.display()
                )
            })?;
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok((RustlsConfig::from_config(Arc::new(config)), leaf_spki))
}
