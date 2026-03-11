# ============================================================
# Prompter — Development & CI Commands
# Usage: make [target]
# ============================================================

# ── Colors ────────────────────────────────────────────────────
YELLOW  := \033[1;33m
GREEN   := \033[0;32m
RED     := \033[0;31m
BLUE    := \033[0;34m
CYAN    := \033[0;36m
NC      := \033[0m

# ── Config ────────────────────────────────────────────────────
export PATH := $(HOME)/.cargo/bin:$(PATH)
CARGO   := cd src-tauri && cargo
NPM     := npm

.PHONY: help setup install \
        dev build \
        test test-rust test-frontend \
        fmt lint check clippy \
        docker-build docker-test docker-check docker-ci \
        clean clean-all clean-docker \
        info status

# ── Default ───────────────────────────────────────────────────
help: ## Show this help
	@echo "$(BLUE)Prompter — Development & CI Commands$(NC)"
	@echo ""
	@echo "$(YELLOW)SETUP:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*SETUP/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)DEVELOPMENT:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*DEV/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)TESTING:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*TEST/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)CODE QUALITY:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*QUALITY/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)DOCKER:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*DOCKER/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)CLEANUP:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*CLEAN/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)INFO:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## .*INFO/ {printf "  $(GREEN)%-22s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ============================================================
# SETUP
# ============================================================

setup: ## [SETUP] One-command dev environment setup
	@bash scripts/setup.sh

install: ## [SETUP] Install npm + Rust dependencies
	@echo "$(BLUE)Installing dependencies...$(NC)"
	$(NPM) install
	$(CARGO) fetch
	@echo "$(GREEN)Dependencies installed$(NC)"

# ============================================================
# DEVELOPMENT
# ============================================================

dev: ## [DEV] Start Tauri development server
	@echo "$(BLUE)Starting Tauri dev server...$(NC)"
	$(NPM) run tauri dev

build: ## [DEV] Build release binary (.dmg)
	@echo "$(BLUE)Building Tauri release...$(NC)"
	$(NPM) run tauri build
	@echo "$(GREEN)Build complete$(NC)"

# ============================================================
# TESTING
# ============================================================

test: ## [TEST] Run all Rust tests
	@echo "$(BLUE)Running Rust tests...$(NC)"
	$(CARGO) test -- --nocapture --test-threads=1
	@echo "$(GREEN)Tests passed$(NC)"

test-rust: test ## [TEST] Alias for test

test-frontend: ## [TEST] TypeScript type check + frontend build
	@echo "$(BLUE)Building frontend...$(NC)"
	$(NPM) run build
	@echo "$(GREEN)Frontend build passed$(NC)"

# ============================================================
# CODE QUALITY
# ============================================================

fmt: ## [QUALITY] Format Rust code
	@echo "$(BLUE)Formatting...$(NC)"
	$(CARGO) fmt
	@echo "$(GREEN)Formatted$(NC)"

fmt-check: ## [QUALITY] Check Rust formatting (CI)
	$(CARGO) fmt --check

lint: ## [QUALITY] Run clippy linter
	@echo "$(BLUE)Linting...$(NC)"
	$(CARGO) clippy -- -D warnings
	@echo "$(GREEN)No lint issues$(NC)"

clippy: lint ## [QUALITY] Alias for lint

check: ## [QUALITY] Cargo check + frontend build
	@echo "$(BLUE)Running checks...$(NC)"
	$(CARGO) check
	$(NPM) run build
	@echo "$(GREEN)All checks passed$(NC)"

# ============================================================
# DOCKER
# ============================================================

docker-build: ## [DOCKER] Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker compose build
	@echo "$(GREEN)Docker image built$(NC)"

docker-test: ## [DOCKER] Run tests in Docker
	@echo "$(BLUE)Running tests in Docker...$(NC)"
	docker compose run --rm dev
	@echo "$(GREEN)Docker tests passed$(NC)"

docker-check: ## [DOCKER] Run cargo check in Docker
	@echo "$(BLUE)Running cargo check in Docker...$(NC)"
	docker compose run --rm dev bash -c "cd src-tauri && cargo check"
	@echo "$(GREEN)Docker check passed$(NC)"

docker-ci: ## [DOCKER] Full CI pipeline in Docker
	@echo "$(BLUE)Running full CI in Docker...$(NC)"
	docker compose run --rm ci
	@echo "$(GREEN)Docker CI passed$(NC)"

# ============================================================
# CLEANUP
# ============================================================

clean: ## [CLEAN] Remove build artifacts (dist/)
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	rm -rf dist/
	@echo "$(GREEN)Clean$(NC)"

clean-all: ## [CLEAN] Remove dist/, node_modules/, and target/
	@echo "$(BLUE)Cleaning everything...$(NC)"
	rm -rf dist/ node_modules/ src-tauri/target/
	@echo "$(GREEN)Full clean complete$(NC)"

clean-docker: ## [CLEAN] Remove Docker images and volumes
	@echo "$(BLUE)Cleaning Docker resources...$(NC)"
	docker compose down -v --rmi local
	@echo "$(GREEN)Docker resources removed$(NC)"

# ============================================================
# INFO
# ============================================================

info: ## [INFO] Show tool versions
	@echo "$(BLUE)Prompter Development Environment$(NC)"
	@echo ""
	@echo "$(YELLOW)Tools:$(NC)"
	@echo "  Rust:   $$(rustc --version 2>/dev/null || echo 'not installed')"
	@echo "  Cargo:  $$(cargo --version 2>/dev/null || echo 'not installed')"
	@echo "  Node:   $$(node --version 2>/dev/null || echo 'not installed')"
	@echo "  npm:    $$(npm --version 2>/dev/null || echo 'not installed')"
	@echo "  cmake:  $$(cmake --version 2>/dev/null | head -1 || echo 'not installed')"
	@echo "  Docker: $$(docker --version 2>/dev/null | cut -d' ' -f3 | tr -d ',' || echo 'not installed')"
	@echo ""
	@echo "$(YELLOW)Project:$(NC)"
	@echo "  Tauri:  $$(grep 'version' src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)"
	@echo "  Node:   $$(node -e "console.log(require('./package.json').version)" 2>/dev/null)"

status: ## [INFO] Show git status
	@git status --short
