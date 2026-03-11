#!/usr/bin/env bash
set -euo pipefail

# ── Colors ──────────────────────────────────────────────────
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Setting up Prompter development environment...${NC}"
echo ""

# ── macOS check ─────────────────────────────────────────────
if [[ "$(uname)" != "Darwin" ]]; then
    echo -e "${RED}Error: Prompter currently supports macOS only.${NC}"
    exit 1
fi

# ── Homebrew ────────────────────────────────────────────────
if ! command -v brew &>/dev/null; then
    echo -e "${YELLOW}Installing Homebrew...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
else
    echo -e "${GREEN}Homebrew found${NC}"
fi

# ── cmake ───────────────────────────────────────────────────
if ! command -v cmake &>/dev/null; then
    echo -e "${YELLOW}Installing cmake...${NC}"
    brew install cmake
else
    echo -e "${GREEN}cmake found: $(cmake --version | head -1)${NC}"
fi

# ── Rust ────────────────────────────────────────────────────
if ! command -v rustc &>/dev/null; then
    echo -e "${YELLOW}Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo -e "${GREEN}Rust found: $(rustc --version)${NC}"
fi

# ── Node.js ─────────────────────────────────────────────────
if ! command -v node &>/dev/null; then
    echo -e "${YELLOW}Installing Node.js...${NC}"
    brew install node
else
    echo -e "${GREEN}Node.js found: $(node --version)${NC}"
fi

# ── npm dependencies ────────────────────────────────────────
echo -e "${BLUE}Installing npm dependencies...${NC}"
cd "$(dirname "$0")/.."
npm install

# ── Rust dependencies ───────────────────────────────────────
echo -e "${BLUE}Fetching Rust dependencies...${NC}"
cd src-tauri
cargo fetch

# ── Done ────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo -e "Next steps:"
echo -e "  ${BLUE}make dev${NC}     — Start development server"
echo -e "  ${BLUE}make test${NC}    — Run tests"
echo -e "  ${BLUE}make help${NC}    — Show all commands"
