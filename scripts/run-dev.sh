#!/usr/bin/env bash
set -e

echo "🚀 Starting in DEV mode (proxy to React dev server)..."

# Run Rust backend without UI embedding
cargo run -p core
