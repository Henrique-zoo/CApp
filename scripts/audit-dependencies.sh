#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

run_backend=false
run_mobile=false

if [ "$#" -eq 0 ]; then
    run_backend=true
    run_mobile=true
fi

while [ "$#" -gt 0 ]; do
    case "$1" in
        --backend) run_backend=true; shift ;;
        --mobile) run_mobile=true; shift ;;
        --all) run_backend=true; run_mobile=true; shift ;;
        --help)
            echo "Uso: $0 [--backend] [--mobile] [--all]"
            exit 0
            ;;
        *)
            echo "Opção desconhecida: $1"
            exit 1
            ;;
    esac
done

if [ "$run_backend" = true ]; then
    echo "================================================================"
    echo "[Backend] Auditoria de Dependências Rust (Cargo / RustSec)"
    echo "================================================================"
    cd "$ROOT_DIR/backend"
    cargo audit --deny warnings
    echo "[SUCESSO] Auditoria do backend concluída sem violações."
fi

if [ "$run_mobile" = true ]; then
    echo "================================================================"
    echo "[Mobile] Auditoria de Dependências Flutter / Dart"
    echo "================================================================"
    cd "$ROOT_DIR/mobile"
    if command -v osv-scanner >/dev/null 2>&1; then
        osv-scanner --config=osv-scanner.toml --lockfile=pubspec.lock
        echo "[SUCESSO] Auditoria do mobile concluída."
    else
        echo "[INFO] osv-scanner não instalado localmente. Verificação será executada no CI."
    fi
fi
