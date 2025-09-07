#!/usr/bin/env bash
set -e

echo "📦 Building React app..."
(
  cd ui && npm run build
)

echo "🚀 Starting in PROD mode (embedded UI)..."

# Run Rust backend with embed-ui feature
cargo run -p core --release --features embed-ui
