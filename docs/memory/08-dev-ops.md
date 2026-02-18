# Dev and Ops

## Local dev
- UI dev server: `npm run dev` from `reauth/ui` (Vite on `http://localhost:5173`).
- UI proxy target override: set `VITE_API_PROXY_TARGET` (defaults to `http://localhost:3000`).
- Backend run: `cargo run --package reauth_core --bin reauth_core` (API on `http://127.0.0.1:3000`).
- Embed UI mode: `npm run build` from `reauth/ui`, then `cargo run --package reauth_core --features embed-ui`.

## Migrations
- Migrations are applied on backend startup via `sqlx::migrate!`.
- One-shot migration run (then exit):
  - `cargo run --package reauth_core --bin reauth_core -- --benchmark`

## Config and environment
- Default config: `reauth/config/default.toml`.
- Overrides via environment variables using `REAUTH__` prefix.
- Examples:
  - `REAUTH__SERVER__PORT=4000`
  - `REAUTH__DATABASE__URL=sqlite:data/reauth.db`

## Database
- Default URL: `sqlite:data/reauth.db`.
- `database.data_dir` defaults to `./data` when not set in config.

## UI delivery modes
- Dev: backend proxies UI requests to `ui.dev_url` when `embed-ui` is disabled.
- Embedded: backend serves `ui/dist` via `rust-embed` when `embed-ui` is enabled.

## Plugins
- Default plugins directory: `reauth/plugins` (dev).
- In production, plugins directory is a sibling of the executable.

## Notes
- Cookies are currently set with `secure=false` for localhost dev.
