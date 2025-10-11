#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo >/dev/null 2>&1; then
  cat <<MSG
[toolchain] cargo not found. Install Rust nightly with:
  rustup toolchain install nightly --component rust-src rustfmt clippy
  rustup default nightly
After installation re-run this script or invoke 'just build'.
MSG
  exit 1
fi

if ! command -v rustup >/dev/null 2>&1; then
  echo "[toolchain] rustup not detected; install from https://rustup.rs" >&2
  exit 1
fi

echo "Rust toolchain detected: $(cargo --version)"
