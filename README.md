<div align="center">
  <img src="ui/public/reauth.svg" alt="ReAuth" width="120" height="120" />

  # ReAuth

  **A modern, high‑performance, single‑binary IdP in Rust + React**

  Multi‑realm · Flow Builder · OIDC/SSO · RBAC · gRPC Plugin POC
</div>

---

## What is ReAuth?
ReAuth is a lightweight identity provider inspired by Keycloak, designed for fast startup, minimal footprint, and a clean architecture. It ships as a single Rust binary that can optionally embed the React UI, or run the UI separately for rapid development.

## Features (current)
- Multi‑realm identity management
- OIDC Authorization Code + PKCE (basic implementation)
- SSO via refresh‑token cookie
- Graph‑based flow builder (React Flow)
- Basic RBAC (roles, permissions, groups)
- gRPC plugin system (proof‑of‑concept)
- SQLite‑first persistence

## Architecture
- Backend: Hexagonal architecture (ports/adapters)
- UI: Feature‑Sliced Design (FSD)
- Styling: shadcn/ui + Tailwind

For deeper documentation, see `docs/memory/`.

---

## Quick start (local dev)

### 1) Backend (API)
```bash
cargo run --package reauth_core --bin reauth_core
```
API runs at: `http://127.0.0.1:3000`

### 2) UI (hot reload)
```bash
cd ui
npm install
npm run dev
```
UI runs at: `http://localhost:5173`

---

## Embed UI (single binary)
```bash
cd ui
npm install
npm run build

# back to repo root
cd ..
cargo run --package reauth_core --features embed-ui
```

---

## Config
Default config lives at `config/default.toml`.
Config precedence (low → high): embedded defaults → `config/default.toml` (dev) → `reauth.toml` / `--config` / `REAUTH_CONFIG` → env.

Env overrides use the `REAUTH__` prefix with `__` separators:

```bash
REAUTH__SERVER__PORT=4000
REAUTH__DATABASE__URL=sqlite:data/reauth.db
```

Example `reauth.toml` for CORS:

```toml
[cors]
allowed_origins = ["http://localhost:5173"]
```

You can also place a `reauth.toml` beside the executable or pass `--config /path/to/reauth.toml`.
`server.public_url` (if set) drives defaults for `auth.issuer` and the default OIDC client URLs.
The default OIDC client (`reauth-admin`) is auto‑synced from config on startup.

---

## CLI Flags
`reauth_core` supports a small set of flags:

- `--benchmark`: run initialization and migrations, then exit (used to validate startup).
- `--config <path>`: load config from a specific file instead of relying on `reauth.toml`.
- `--print-config`: print the resolved config (with secrets redacted) and exit.

---

## Database & migrations
Migrations are applied automatically on startup. To run migrations and exit:

```bash
cargo run --package reauth_core --bin reauth_core -- --benchmark
```

Default DB: `sqlite:data/reauth.db`

---

## Plugins (POC)
Plugins are discovered in `plugins/` and managed via the API:
- `GET /api/plugins/manifests`
- `POST /api/plugins/{id}/enable`
- `POST /api/plugins/{id}/disable`

---

## Project structure (high‑level)
```
reauth/
├─ crates/reauth_core/         # Rust backend
├─ ui/                         # React UI
├─ migrations/                 # SQLite schema
├─ plugins/                    # Plugin binaries + manifests
├─ proto/                      # gRPC proto files
└─ docs/memory/                # Architecture + flow docs
```

---

## Roadmaps & docs
- Memory docs: `docs/memory/`
- Feature roadmaps: `docs/memory/roadmaps/`

---

## License
TBD
