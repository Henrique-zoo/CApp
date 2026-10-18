#!/usr/bin/env bash
# ==============================================================================
# Script de Auditoria Local de Vulnerabilidades e Dependências — CApp
# ==============================================================================
# Uso:
#   ./scripts/audit-dependencies.sh [OPÇÕES]
#
# Opções:
#   -a, --all        Executa a varredura completa (Backend e Mobile) [Padrão]
#   -b, --backend    Executa a auditoria apenas no Backend (Rust / Cargo)
#   -m, --mobile     Executa a auditoria apenas no Mobile (Flutter / Dart)
#   -u, --update     Verifica dependências desatualizadas sem aplicar mudanças
#   -h, --help       Exibe esta mensagem de ajuda
# ==============================================================================

set -euo pipefail

# Cores para terminal
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  RED=''
  GREEN=''
  YELLOW=''
  BLUE=''
  BOLD=''
  NC=''
fi

# Localização do monorepo
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TARGET="all"
CHECK_UPDATES=false

show_help() {
  echo -e "${BOLD}Script de Auditoria Local de Dependências — Monorepo CApp${NC}"
  echo -e ""
  echo -e "${BOLD}USO:${NC}"
  echo -e "  bash scripts/audit-dependencies.sh [OPÇÕES]"
  echo -e ""
  echo -e "${BOLD}OPÇÕES:${NC}"
  echo -e "  -a, --all        Executa auditoria em todos os módulos (Backend e Mobile) [Padrão]"
  echo -e "  -b, --backend    Executa auditoria apenas no Backend (Rust / Cargo)"
  echo -e "  -m, --mobile     Executa auditoria apenas no Mobile (Flutter / Dart)"
  echo -e "  -u, --update     Verifica versões mais recentes disponíveis (modo informativo)"
  echo -e "  -h, --help       Exibe esta tela de ajuda"
  echo -e ""
  echo -e "${BOLD}EXEMPLOS:${NC}"
  echo -e "  ./scripts/audit-dependencies.sh"
  echo -e "  ./scripts/audit-dependencies.sh --backend"
  echo -e "  ./scripts/audit-dependencies.sh --mobile"
  echo -e "  ./scripts/audit-dependencies.sh --update"
}

# Processamento de argumentos
while [[ $# -gt 0 ]]; do
  case "$1" in
    -a|--all)
      TARGET="all"
      shift
      ;;
    -b|--backend)
      TARGET="backend"
      shift
      ;;
    -m|--mobile)
      TARGET="mobile"
      shift
      ;;
    -u|--update)
      CHECK_UPDATES=true
      shift
      ;;
    -h|--help)
      show_help
      exit 0
      ;;
    *)
      echo -e "${RED}[ERRO] Opção desconhecida: $1${NC}"
      show_help
      exit 1
      ;;
  esac
done

BACKEND_DIR="${REPO_ROOT}/backend"
MOBILE_DIR="${REPO_ROOT}/mobile"

EXIT_CODE=0

# ==============================================================================
# Auditoria de Dependências: Backend (Rust / Cargo)
# ==============================================================================
audit_backend() {
  echo -e "\n${BLUE}================================================================${NC}"
  echo -e "${BLUE}${BOLD}[Backend] Auditoria de Dependências Rust (Cargo / RustSec)${NC}"
  echo -e "${BLUE}================================================================${NC}"

  if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}[ERRO] O binário 'cargo' não foi encontrado no PATH.${NC}"
    echo -e "Instale a toolchain oficial do Rust em https://rustup.rs."
    return 1
  fi

  if ! cargo audit --version >/dev/null 2>&1; then
    echo -e "${YELLOW}[AVISO] 'cargo-audit' não está instalado na toolchain local.${NC}"
    echo -e "Para instalar localmente execute:"
    echo -e "  ${BOLD}cargo install --locked cargo-audit${NC}\n"
    echo -e "Continuando verificação sintática e lockfile..."
    (cd "${BACKEND_DIR}" && cargo check --locked)
    return 0
  fi

  echo -e "${BOLD}Executando varredura contra RustSec Advisory Database...${NC}"
  if (cd "${BACKEND_DIR}" && cargo audit --deny warnings); then
    echo -e "${GREEN}[OK] Nenhuma vulnerabilidade ou crate descontinuado encontrado no Backend.${NC}"
  else
    echo -e "${RED}[FALHA] Foram identificadas vulnerabilidades ou inconsistências nas dependências do Backend.${NC}"
    EXIT_CODE=1
  fi

  if [ "${CHECK_UPDATES}" = true ]; then
    echo -e "\n${BOLD}Checando atualizações disponíveis de crates (dry-run):${NC}"
    (cd "${BACKEND_DIR}" && cargo update --dry-run || true)
  fi
}

# ==============================================================================
# Auditoria de Dependências: Mobile (Flutter / Dart)
# ==============================================================================
audit_mobile() {
  echo -e "\n${BLUE}================================================================${NC}"
  echo -e "${BLUE}${BOLD}[Mobile] Auditoria de Dependências Flutter/Dart (OSV / pub)${NC}"
  echo -e "${BLUE}================================================================${NC}"

  local FLUTTER_CMD=""
  if command -v flutter >/dev/null 2>&1; then
    FLUTTER_CMD="flutter"
  elif command -v dart >/dev/null 2>&1; then
    FLUTTER_CMD="dart"
  fi

  if [ -n "${FLUTTER_CMD}" ]; then
    echo -e "${BOLD}Verificando integridade e dependências desatualizadas no Mobile...${NC}"
    if [ "${FLUTTER_CMD}" = "flutter" ]; then
      (cd "${MOBILE_DIR}" && flutter pub get --enforce-lockfile)
      (cd "${MOBILE_DIR}" && flutter pub outdated)
    else
      (cd "${MOBILE_DIR}" && dart pub get --enforce-lockfile)
      (cd "${MOBILE_DIR}" && dart pub outdated)
    fi
  else
    echo -e "${YELLOW}[AVISO] 'flutter' ou 'dart' não encontrados no PATH local.${NC}"
    echo -e "A integridade do lockfile e análise pub serão validadas no pipeline de CI."
  fi

  if command -v osv-scanner >/dev/null 2>&1; then
    echo -e "\n${BOLD}Executando varredura com OSV-Scanner em mobile/pubspec.lock...${NC}"
    if osv-scanner --lockfile="${MOBILE_DIR}/pubspec.lock" --config="${MOBILE_DIR}/osv-scanner.toml"; then
      echo -e "${GREEN}[OK] Nenhuma vulnerabilidade conhecida encontrada em mobile/pubspec.lock.${NC}"
    else
      echo -e "${RED}[FALHA] Vulnerabilidades detectadas pelo OSV-Scanner no Mobile.${NC}"
      EXIT_CODE=1
    fi
  else
    echo -e "\n${YELLOW}[INFO] 'osv-scanner' não instalado localmente.${NC}"
    echo -e "Para executar a varredura OSV localmente, instale via:"
    echo -e "  go install github.com/google/osv-scanner/cmd/osv-scanner@latest"
    echo -e "  (ou baixe o binário em https://github.com/google/osv-scanner/releases)"
    echo -e "A esteira GitHub Actions executará a verificação automaticamente via CI."
  fi
}

# Execução conforme o alvo
case "${TARGET}" in
  backend)
    audit_backend
    ;;
  mobile)
    audit_mobile
    ;;
  all)
    audit_backend
    audit_mobile
    ;;
esac

echo -e "\n${BLUE}================================================================${NC}"
if [ ${EXIT_CODE} -eq 0 ]; then
  echo -e "${GREEN}${BOLD}[SUCESSO] Auditoria de dependências concluída sem violações.${NC}"
else
  echo -e "${RED}${BOLD}[ATENÇÃO] A auditoria de dependências identificou problemas a corrigir.${NC}"
fi
echo -e "${BLUE}================================================================${NC}"

exit ${EXIT_CODE}
