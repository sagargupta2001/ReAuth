#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

CONFORMANCE_SUITE_DIR=${CONFORMANCE_SUITE_DIR:-"$ROOT_DIR/.tmp/conformance-suite"}
CONFORMANCE_SUITE_REF=${CONFORMANCE_SUITE_REF:-"release-v5.1.35"}
CONFORMANCE_ALIAS=${CONFORMANCE_ALIAS:-"reauth"}
CONFORMANCE_BASE_URL=${CONFORMANCE_BASE_URL:-"https://localhost.emobix.co.uk:8443/"}
CONFORMANCE_MTLS_URL=${CONFORMANCE_MTLS_URL:-"https://localhost.emobix.co.uk:8444/"}
REAUTH_BASE_URL=${REAUTH_BASE_URL:-""}
REAUTH_WAIT_URL=${REAUTH_WAIT_URL:-"http://localhost:3000"}
REAUTH_REALM=${REAUTH_REALM:-"master"}
OIDC_CLIENT_ID=${OIDC_CLIENT_ID:-"reauth-conformance"}
ADMIN_USERNAME=${ADMIN_USERNAME:-"admin"}
ADMIN_PASSWORD=${ADMIN_PASSWORD:-"admin"}

TEMPLATE_FILE="$ROOT_DIR/tests/oidc_conformance/reauth-basic-ci.json.tmpl"
CONFIG_OUT="$ROOT_DIR/.tmp/reauth-basic-ci.json"
REAUTH_CONFIG_FILE="$ROOT_DIR/tests/oidc_conformance/reauth-conformance.toml"
REAUTH_LOG="$ROOT_DIR/.tmp/reauth-conformance.log"

cleanup() {
  local exit_code=$?
  if [[ -n "${REAUTH_PID:-}" ]]; then
    kill "$REAUTH_PID" >/dev/null 2>&1 || true
  fi
  if [[ -d "$CONFORMANCE_SUITE_DIR" ]]; then
    (cd "$CONFORMANCE_SUITE_DIR" && docker compose down -v) || true
  fi
  exit "$exit_code"
}
trap cleanup EXIT

mkdir -p "$ROOT_DIR/.tmp"

if [[ -z "$REAUTH_BASE_URL" ]]; then
  if command -v getent >/dev/null 2>&1 && getent hosts host.docker.internal >/dev/null 2>&1; then
    REAUTH_BASE_URL="http://host.docker.internal:3000"
  else
    GATEWAY=$(docker network inspect bridge -f '{{(index .IPAM.Config 0).Gateway}}' 2>/dev/null || true)
    if [[ -n "$GATEWAY" ]]; then
      REAUTH_BASE_URL="http://$GATEWAY:3000"
    else
      REAUTH_BASE_URL="http://host.docker.internal:3000"
    fi
  fi
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker CLI not found. Please install Docker Desktop and ensure 'docker' is in PATH."
  exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "Docker Compose is not available. Ensure Docker Desktop is running and 'docker compose' works."
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "Docker daemon is not running. Start Docker Desktop and try again."
  exit 1
fi

if [[ ! -d "$CONFORMANCE_SUITE_DIR/.git" ]]; then
  git clone https://gitlab.com/openid/conformance-suite.git "$CONFORMANCE_SUITE_DIR"
fi

pushd "$CONFORMANCE_SUITE_DIR" >/dev/null
  git fetch --tags >/dev/null
  git checkout "$CONFORMANCE_SUITE_REF" >/dev/null

  export MAVEN_CACHE="$CONFORMANCE_SUITE_DIR/.m2"
  docker compose -f builder-compose.yml run --rm builder
  docker compose up -d
popd >/dev/null

# Build UI + run embedded server so the conformance suite can complete login/consent.
(
  cd "$ROOT_DIR/ui"
  npm ci
  npm run build
)

# Build the ReAuth binary before starting the readiness timer.
# In CI, compiling from scratch can take longer than the readiness budget even when
# the server would otherwise boot correctly once launched.
(
  cd "$ROOT_DIR"
  cargo build --features embed-ui --bin reauth
)

pushd "$ROOT_DIR" >/dev/null
  export REAUTH__SERVER__PUBLIC_URL="$REAUTH_BASE_URL"
  export REAUTH__AUTH__ISSUER="$REAUTH_BASE_URL"
  ./target/debug/reauth --config "$REAUTH_CONFIG_FILE" > "$REAUTH_LOG" 2>&1 &
  REAUTH_PID=$!
  echo "$REAUTH_PID" > "$ROOT_DIR/.tmp/reauth.pid"
popd >/dev/null

# Wait for ReAuth to boot
REAUTH_DISCOVERY_URL="$REAUTH_WAIT_URL/api/realms/$REAUTH_REALM/oidc/.well-known/openid-configuration"
for i in {1..60}; do
  if curl -fsS "$REAUTH_DISCOVERY_URL" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$REAUTH_PID" >/dev/null 2>&1; then
    echo "ReAuth exited before becoming ready. Check $REAUTH_LOG"
    exit 1
  fi
  sleep 2
  if [[ $i -eq 60 ]]; then
    echo "ReAuth did not become ready in time. Check $REAUTH_LOG"
    exit 1
  fi
done

# Render conformance config from template
export TEMPLATE_FILE CONFIG_OUT
export CONFORMANCE_ALIAS REAUTH_BASE_URL REAUTH_REALM OIDC_CLIENT_ID ADMIN_USERNAME ADMIN_PASSWORD
python3 - <<'PY'
import os
from pathlib import Path

template_path = Path(os.environ['TEMPLATE_FILE'])
output_path = Path(os.environ['CONFIG_OUT'])

content = template_path.read_text(encoding='utf-8')
content = os.path.expandvars(content)
output_path.write_text(content, encoding='utf-8')
print(f"Rendered conformance config to {output_path}")
PY

# Install python deps if the suite declares any
if [[ -f "$CONFORMANCE_SUITE_DIR/scripts/requirements.txt" ]]; then
  python3 -m pip install -r "$CONFORMANCE_SUITE_DIR/scripts/requirements.txt"
fi

export PYTHONHTTPSVERIFY=0

# The local Docker-based conformance suite is started in fintechlabs dev mode.
# In that mode, run-test-plan.py intentionally skips API bearer-token auth and
# uses the suite's built-in localhost.emobix.co.uk defaults.
#
# Only force explicit server-mode URLs when the caller also provides a
# CONFORMANCE_TOKEN for an authenticated suite deployment.
if [[ -n "${CONFORMANCE_TOKEN:-}" ]]; then
  export CONFORMANCE_SERVER="$CONFORMANCE_BASE_URL"
  export CONFORMANCE_SERVER_MTLS="$CONFORMANCE_MTLS_URL"
fi

python3 "$CONFORMANCE_SUITE_DIR/scripts/run-test-plan.py" \
  "oidcc-basic-certification-test-plan[server_metadata=discovery][client_registration=static_client]" \
  "$CONFIG_OUT"
