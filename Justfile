# Use bash strict mode
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Read active rust toolchain from mise for display/logging

# Shared env (same as CI)
RUSTFLAGS_BASE := "-Dwarnings -Zshare-generics=y -Zthreads=0"
RUSTDOCFLAGS_BASE := "-Dwarnings -Zshare-generics=y -Zthreads=0"
WASM_TARGET := "wasm32-unknown-unknown"

# Default: run everything
default:
    @just --list

# Install system libraries used by CI (Ubuntu/Debian)
deps:
	@sudo apt-get update
	@sudo apt-get install --no-install-recommends -y libasound2-dev libudev-dev libwayland-dev

# Format check
fmt:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}}" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	cargo fmt --all -- --check

# Docs check
docs:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}}" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	cargo doc --locked --workspace --profile ci --all-features --document-private-items --no-deps

# Clippy lints
clippy:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}}" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	cargo clippy --locked --workspace --all-targets --profile ci --all-features

# Bevy lints (requires bevy_lint on PATH)
bevy-lints:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}}" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	bevy_lint --locked --workspace --all-targets --profile ci --all-features

# Install Bevy linter via the Bevy CLI installer, then ensure bevy_lint exists
bevy-lint-install:
	@curl -sSfL https://raw.githubusercontent.com/TheBevyFlock/bevy_cli/main/install.sh | bash   # installs the CLI wrapper [web:2]
	@bevy lint install v0.4.0 || true                                                             # pins CLI linter like CI [web:1]
	@command -v bevy_lint >/dev/null 2>&1 || { echo "bevy_lint not on PATH; ensure installer completed."; exit 1; }

# Tests with cranelift backend
test:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}} -Zcodegen-backend=cranelift" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	cargo test --locked --workspace --all-targets --profile ci --no-fail-fast

# Web compilation check with getrandom wasm cfg injection
check-web:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}} --cfg getrandom_backend=\"wasm_js\"" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	cargo check \
	  --config 'profile.web.inherits="dev"' \
	  --profile ci \
	  --no-default-features \
	  --features dev \
	  --target {{WASM_TARGET}}

# Run everything in CI order
all: fmt docs clippy bevy-lints test check-web

# Clean
clean:
	@cargo clean

# Run native via Bevy CLI
run:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}}" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	bevy run  # builds and runs desktop app [web:17]

# Run web via Bevy CLI
# Bevy CLI handles building to wasm32-unknown-unknown and serving locally.
run-web:
	@env \
	RUSTFLAGS="{{RUSTFLAGS_BASE}}" \
	RUSTDOCFLAGS="{{RUSTDOCFLAGS_BASE}}" \
	bevy run web  # serves on localhost, typically port 4000 [web:1][web:17]
