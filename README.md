<div align="center">
  <img src="ui/public/reauth.svg" alt="ReAuth" width="120" height="120" />

  # ReAuth

  **A modern, high‑performance, single‑binary IdP in Rust + React**

  Multi‑realm · Flow Builder · OIDC/SSO · RBAC
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
make dev
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
make embed
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

List env vars (like `cors.allowed_origins`) use comma‑separated values:

```bash
REAUTH__CORS__ALLOWED_ORIGINS=http://localhost:5173,http://localhost:4010
```

Logging can be tuned via config (or `RUST_LOG` for advanced filtering):

```toml
[logging]
level = "info"
filter = "reauth_core=info,sqlx=warn"
```

You can also place a `reauth.toml` beside the executable or pass `--config /path/to/reauth.toml`.
`server.public_url` (if set) drives defaults for `auth.issuer` and the default OIDC client URLs.
The default OIDC client (`reauth-admin`) is auto‑synced from config on startup.
Builds generate a commented `reauth.toml` template next to the binary if one does not already exist.
If a config file is present, changes are hot‑reloaded at runtime (note: bind address/port, DB path, and JWT settings still require a restart).
You can also trigger a manual reload via API (requires `realm:write`):

```bash
curl -X POST http://localhost:3000/api/config/reload
```

---

## CLI Flags
`reauth_core` supports a small set of flags:

- `--help`, `-h`: show minimal help and exit (this list will expand as the CLI grows).
- `--benchmark`: run initialization and migrations, then exit (used to validate startup).
- `--config <path>`: load config from a specific file instead of relying on `reauth.toml`.
- `--print-config`: print the resolved config (with secrets redacted) and exit.
- `--init-config`: write a commented `reauth.toml` template next to the binary and exit.
- `--check-config`: validate resolved config and exit.
- `--seed-only`: run migrations + seeding, then exit.
- `--seed-status`: print applied seeders and exit.

Note for automation/LLMs: prefer `--help` and this section as the source of truth for supported flags.

Examples:

```bash
./reauth_core --print-config
./reauth_core --init-config
./reauth_core --check-config
./reauth_core --config /path/to/reauth.toml --print-config
./reauth_core --seed-only
./reauth_core --seed-status
./reauth_core --benchmark
```

---

## Database & migrations
Migrations are applied automatically on startup. To run migrations and exit:

```bash
cargo run --package reauth_core --bin reauth_core -- --benchmark
```

Default DB: `sqlite:data/reauth.db`

---

## Project structure (high‑level)
```
reauth/
├─ crates/reauth_core/         # Rust backend
├─ ui/                         # React UI
├─ migrations/                 # SQLite schema
└─ docs/memory/                # Architecture + flow docs
```

---

## Roadmaps & docs
- Memory docs: `docs/memory/`
- Feature roadmaps: `docs/memory/roadmaps/`
- Webhooks event engine roadmap: `docs/memory/roadmaps/webhooks.md`

---

## License
TBD
