//! Testes de subprocesso para verificação de diagnósticos seguros e não vazamento de segredos.
//!
//! Executa o binário do backend como subprocesso sob diferentes configurações inválidas,
//! inspecionando stdout e stderr para garantir a ausência de vazamento de credenciais.

use std::process::Command;

#[test]
fn test_subprocess_malformed_query_param_does_not_leak_secrets() {
    let fake_password = "SENHA_FICTICIA_MALFORMED_12345";
    let fake_user = "user_sigiloso_teste";
    let fake_param = "passwrod";
    let sensitive_url = format!(
        "postgres://{fake_user}:{fake_password}@127.0.0.1:5432/capp?{fake_param}={fake_password}"
    );

    let output = Command::new(env!("CARGO_BIN_EXE_backend"))
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("DATABASE_URL", &sensitive_url)
        .env("FIREBASE_PROJECT_ID", "capp-test-project")
        .env("APP_ENV", "development")
        .output()
        .expect("Falha ao executar o binário do backend como subprocesso");

    assert!(
        !output.status.success(),
        "O processo do backend deveria falhar na inicialização com URL malformada"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stderr.contains(fake_password),
        "A senha fictícia vazou no stderr: {stderr}"
    );
    assert!(
        !stdout.contains(fake_password),
        "A senha fictícia vazou no stdout: {stdout}"
    );
    assert!(
        !stderr.contains(fake_param),
        "O parâmetro desconhecido vazou no stderr: {stderr}"
    );
    assert!(
        !stderr.contains(fake_user),
        "O usuário do banco vazou no stderr: {stderr}"
    );

    assert!(
        stderr.contains(
            "a URL de conexão contém parâmetros de consulta não permitidos ou desconhecidos"
        ),
        "O stderr deveria conter diagnóstico seguro sobre parâmetros não permitidos. Obtido: {stderr}"
    );
}

#[test]
fn test_subprocess_invalid_sslmode_does_not_leak_secrets() {
    let fake_password = "SENHA_FICTICIA_SSLMODE_998877";
    let sensitive_url =
        format!("postgres://usr:senhasecreta@127.0.0.1:5432/capp?sslmode={fake_password}");

    let output = Command::new(env!("CARGO_BIN_EXE_backend"))
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("DATABASE_URL", &sensitive_url)
        .env("FIREBASE_PROJECT_ID", "capp-test-project")
        .env("APP_ENV", "development")
        .output()
        .expect("Falha ao executar o binário do backend como subprocesso");

    assert!(
        !output.status.success(),
        "O processo do backend deveria falhar na inicialização com sslmode inválido"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stderr.contains(fake_password),
        "O valor inválido de sslmode (senha) vazou no stderr: {stderr}"
    );
    assert!(
        !stdout.contains(fake_password),
        "O valor inválido de sslmode (senha) vazou no stdout: {stdout}"
    );

    assert!(
        stderr.contains("o parâmetro 'sslmode' na DATABASE_URL contém valor inválido"),
        "O stderr deveria conter diagnóstico seguro sobre valor inválido de sslmode. Obtido: {stderr}"
    );
}

#[test]
fn test_subprocess_production_requires_verify_full() {
    let non_verify_full_url = "postgres://usr:senhasecreta@127.0.0.1:5432/capp?sslmode=require";

    let output = Command::new(env!("CARGO_BIN_EXE_backend"))
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("DATABASE_URL", non_verify_full_url)
        .env("FIREBASE_PROJECT_ID", "capp-test-project")
        .env("APP_ENV", "production")
        .output()
        .expect("Falha ao executar o binário do backend como subprocesso");

    assert!(
        !output.status.success(),
        "O processo do backend deveria falhar em produção sem verify-full"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("o ambiente de produção ('production') exige que a DATABASE_URL configure explicitamente 'sslmode=verify-full'"),
        "O stderr deveria exigir explicitamente sslmode=verify-full em produção. Obtido: {stderr}"
    );
}

#[test]
fn test_subprocess_port_zero_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_backend"))
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env(
            "DATABASE_URL",
            "postgres://usr:senhasecreta@127.0.0.1:5432/capp",
        )
        .env("FIREBASE_PROJECT_ID", "capp-test-project")
        .env("BACKEND_PORT", "0")
        .output()
        .expect("Falha ao executar o binário do backend como subprocesso");

    assert!(
        !output.status.success(),
        "O processo do backend deveria falhar quando BACKEND_PORT=0"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("porta 0 é inválida"),
        "O stderr deveria apontar que a porta 0 é inválida. Obtido: {stderr}"
    );
}
