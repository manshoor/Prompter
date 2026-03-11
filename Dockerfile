# ============================================================
# Prompter CI Dockerfile
# For CI testing only — Tauri dev/build requires native macOS
# ============================================================

# ── Stage: base ─────────────────────────────────────────────
FROM rust:1.94-bookworm AS base

# System dependencies for Tauri (Linux equivalents for compilation/check)
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libasound2-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js 22 LTS
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install rustfmt and clippy
RUN rustup component add rustfmt clippy

WORKDIR /app

# ── Stage: dev ──────────────────────────────────────────────
FROM base AS dev

# Copy frontend package files first (layer caching)
COPY package.json package-lock.json ./
RUN npm ci

# Copy Cargo manifests for dependency caching
COPY src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/
COPY src-tauri/build.rs src-tauri/
COPY src-tauri/capabilities src-tauri/capabilities/
COPY src-tauri/gen src-tauri/gen/
COPY src-tauri/icons src-tauri/icons/

# Create stub source files so cargo fetch works
RUN mkdir -p src-tauri/src && \
    echo "pub fn run() {}" > src-tauri/src/lib.rs && \
    echo "fn main() { prompter_lib::run() }" > src-tauri/src/main.rs

# Fetch and cache Rust dependencies
RUN cd src-tauri && cargo fetch

# Copy full source (invalidates cache only when source changes)
COPY . .

CMD ["bash", "-c", "cd src-tauri && cargo test -- --test-threads=1"]

# ── Stage: ci ───────────────────────────────────────────────
FROM dev AS ci

# Run all CI checks in sequence
CMD ["bash", "-c", "\
    echo '=== cargo check ===' && cd src-tauri && cargo check && \
    echo '=== cargo test ===' && cargo test -- --test-threads=1 && \
    echo '=== cargo fmt --check ===' && cargo fmt --check && \
    echo '=== cargo clippy ===' && cargo clippy -- -D warnings && \
    echo '=== npm run build ===' && cd /app && npm run build && \
    echo '=== All CI checks passed ==='"]
