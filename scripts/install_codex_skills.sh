#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_DIR="$ROOT_DIR/skills/codex"
TARGET_ROOT="${CODEX_HOME:-$HOME/.codex}/skills"

mkdir -p "$TARGET_ROOT"

for skill_dir in "$SOURCE_DIR"/*; do
  [ -d "$skill_dir" ] || continue
  skill_name="$(basename "$skill_dir")"
  ln -sfn "$skill_dir" "$TARGET_ROOT/$skill_name"
  echo "linked $skill_name -> $TARGET_ROOT/$skill_name"
done

echo "Codex skills linked into $TARGET_ROOT"
