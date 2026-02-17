# Control Flow

## Scope
This doc captures the primary control paths and state transitions. It is intentionally high level and stable. Protocol specifics live in `reauth/docs/memory/04-oidc-sso-flows.md`. Flow-builder internals live in `reauth/docs/memory/05-flow-builder.md`.

## Backend startup
```mermaid
sequenceDiagram
  participant Main as main.rs
  participant Init as bootstrap::initialize
  participant DB as Database
  participant Seed as seed_database
  participant Server as web::start_server

  Main->>Init: initialize()
  Init->>DB: init db + run migrations
  Init->>Seed: seed defaults (realm, flows, admin, oidc)
  Init->>Server: start_server(AppState)
```

## Request lifecycle (API)
```mermaid
sequenceDiagram
  participant C as Client
  participant W as Web Adapter (Axum)
  participant A as Application Services
  participant D as Domain
  participant P as Persistence

  C->>W: HTTP request
  W->>A: Validate + map to use case
  A->>D: Execute domain logic
  A->>P: Load/store data
  A-->>W: Response DTO
  W-->>C: HTTP response
```

## Login flow (browser, high level)
Primary entry points:
- Start: `GET /api/realms/{realm}/auth/login`
- Step submit: `POST /api/realms/{realm}/auth/login/execute`

```mermaid
sequenceDiagram
  participant U as User Agent
  participant UI as React UI (AuthFlowExecutor)
  participant API as Auth Handler
  participant Exec as FlowExecutor
  participant Store as FlowStore/AuthSessionRepo

  U->>UI: Open login
  UI->>API: GET /auth/login
  API->>Store: resume or create AuthenticationSession
  API->>Exec: execute(session_id, None)
  Exec-->>API: Challenge | Success | Failure
  API-->>UI: JSON response + login cookie

  UI->>API: POST /auth/login/execute (credentials)
  API->>Exec: execute(session_id, input)
  Exec-->>API: Challenge | Success | Failure
  API-->>UI: JSON response

  alt Success
    API->>API: issue refresh token, or OIDC auth code
    API-->>UI: redirect target
  end
```

## Flow execution state machine
Graph execution is driven by `AuthenticationSession` + `ExecutionPlan`.

```mermaid
stateDiagram-v2
  [*] --> StartNode
  StartNode --> LogicOrAuthNode
  LogicOrAuthNode --> LogicOrAuthNode: Continue
  LogicOrAuthNode --> WaitingForUI: SuspendForUI
  WaitingForUI --> LogicOrAuthNode: handle_input
  LogicOrAuthNode --> TerminalSuccess
  LogicOrAuthNode --> TerminalFailure
  TerminalSuccess --> [*]
  TerminalFailure --> [*]
```

## UI boot and auth gating
- Auth guard checks:
  1. OIDC callback code in URL -> exchange token
  2. If no code, attempt refresh token
  3. If unauthenticated, redirect to `/#/login?redirect=...`
- Login page mounts `AuthFlowExecutor` which drives the flow via API calls.

## UI delivery modes
- Dev mode: API proxies UI routes to React dev server when `embed-ui` is disabled.
- Embedded mode: API serves static assets from the binary when `embed-ui` is enabled.

## Flow catalog (summary)
All auth flows share the same execution engine. Per-flow details live in `reauth/docs/memory/11-auth-flow-catalog.md`.
