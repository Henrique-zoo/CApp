#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

run_backend=false
run_mobile=false
mode_update=false
extra_cargo_audit_flags=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --backend) run_backend=true; shift ;;
        --mobile) run_mobile=true; shift ;;
        --all) run_backend=true; run_mobile=true; shift ;;
        --update) mode_update=true; shift ;;
        --no-fetch) extra_cargo_audit_flags="${extra_cargo_audit_flags} --no-fetch"; shift ;;
        --help)
            echo "Uso: $0 [--backend] [--mobile] [--all] [--update] [--no-fetch]"
            echo ""
            echo "Opções:"
            echo "  --backend   Executa exclusivamente para o backend (Rust)"
            echo "  --mobile    Executa exclusivamente para o mobile (Flutter/Dart)"
            echo "  --all       Executa para backend e mobile (padrão quando nenhum escopo for informado)"
            echo "  --update    Modo informativo: verifica atualizações disponíveis de dependências"
            echo "  --no-fetch  Não atualiza o banco de advisories (execução offline para cargo-audit)"
            echo "  --help      Exibe esta mensagem de ajuda"
            exit 0
            ;;
        *)
            echo "Opção desconhecida: $1" >&2
            echo "Uso: $0 [--backend] [--mobile] [--all] [--update]" >&2
            exit 1
            ;;
    esac
done

if [ "$run_backend" = false ] && [ "$run_mobile" = false ]; then
    run_backend=true
    run_mobile=true
fi

if [ "$mode_update" = true ]; then
    if [ "$run_backend" = true ]; then
        echo "================================================================"
        echo "[Backend] Verificação de Atualizações de Dependências Rust"
        echo "================================================================"
        cd "$ROOT_DIR/backend"
        if command -v cargo-outdated >/dev/null 2>&1; then
            cargo outdated
        else
            cargo update --dry-run
        fi
        echo "[SUCESSO] Verificação de atualizações do backend concluída."
    fi

    if [ "$run_mobile" = true ]; then
        echo "================================================================"
        echo "[Mobile] Verificação de Atualizações de Dependências Flutter/Dart"
        echo "================================================================"
        cd "$ROOT_DIR/mobile"
        if command -v flutter >/dev/null 2>&1; then
            flutter pub outdated
        elif command -v dart >/dev/null 2>&1; then
            dart pub outdated
        else
            echo "[ERRO] 'flutter' ou 'dart' não foi encontrado no PATH para verificar dependências desatualizadas." >&2
            exit 1
        fi
        echo "[SUCESSO] Verificação de atualizações do mobile concluída."
    fi
    exit 0
fi

if [ "$run_backend" = true ]; then
    echo "================================================================"
    echo "[Backend] Auditoria de Dependências Rust (Cargo / RustSec)"
    echo "================================================================"
    cd "$ROOT_DIR/backend"
    if ! command -v cargo-audit >/dev/null 2>&1 && ! cargo audit --version >/dev/null 2>&1; then
        echo "[ERRO] cargo-audit não está instalado no ambiente." >&2
        echo "Para instalar: cargo install cargo-audit" >&2
        exit 1
    fi
    cargo audit ${extra_cargo_audit_flags}
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
        echo "[ERRO] osv-scanner não está instalado no ambiente." >&2
        echo "Para instalar o osv-scanner:" >&2
        echo "  - Via go: go install github.com/google/osv-scanner/cmd/osv-scanner@latest" >&2
        echo "  - Via cargo: cargo install osv-scanner" >&2
        echo "  - Ou consulte as instruções em: https://github.com/google/osv-scanner#installation" >&2
        exit 1
    fi
fi
