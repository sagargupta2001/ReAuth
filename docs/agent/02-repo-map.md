# Repository Map

Use this file to orient yourself before editing code.

## Top-level layout

- `src/`: Rust backend
- `ui/`: React frontend
- `migrations/`: SQLite schema migrations
- `tests/`: backend integration and contract-style tests
- `docs/memory/`: architecture, product memory, and roadmaps
- `docs/agent/`: required implementation guidance and workflow
- `specs/`: feature specs used for spec-driven development

## Backend map

### `src/domain`

Core business model and stable concepts:

- entities and value objects
- flow node metadata and execution contracts
- protocol/domain rules
- compile-time and runtime flow primitives

Do not put delivery, SQL, Axum, or infrastructure concerns here.

### `src/application`

Use-case orchestration and cross-domain coordination:

- auth, OIDC, flows, themes, RBAC, webhooks
- executor/manager/service logic
- script engine abstraction
- policy, diagnostics, and helper orchestration

This layer depends on domain + ports, not concrete adapters.

### `src/ports`

Trait boundaries for infrastructure and external concerns:

- repositories
- token services
- cache
- transaction manager
- eventing

### `src/adapters`

Concrete implementations:

- `web/`: Axum handlers, routing, middleware
- `persistence/`: SQLite repositories and storage
- `auth/`: runtime node workers and auth executors
- `crypto/`, `cache/`, `eventing/`, `logging/`, `observability/`

### `src/bootstrap`

Startup wiring:

- config loading
- infra initialization
- service construction
- seeding
- app state composition

## Frontend map

### `ui/src/app`

Application shell, providers, routing, bootstrapping.

### `ui/src/pages`

Route-level page assemblies.

### `ui/src/widgets`

Reusable page sections that compose multiple features/entities.

### `ui/src/features`

Feature-level behavior, hooks, API access, forms, and components.

Put new UI behavior here first unless it is clearly an entity or shared concern.

### `ui/src/entities`

Stable domain-facing UI models and entity utilities.

### `ui/src/shared`

Cross-cutting reusable infrastructure:

- API client
- query keys
- shared hooks
- shared UI primitives
- test setup

## Where to start when changing something

- Existing auth journey:
  - backend: `src/adapters/web/auth_handler.rs`, `src/application/flow_executor/`, `src/adapters/auth/`
  - frontend: `ui/src/features/auth/`
- Flow builder:
  - backend: `src/application/flow_manager/`, `src/application/node_registry.rs`
  - frontend: `ui/src/features/flow-builder/`, `ui/src/entities/flow/`
- Themes/fluid:
  - backend: `src/adapters/web/theme_handler.rs`, `src/domain/theme_pages.rs`
  - frontend: `ui/src/features/theme/`, `ui/src/features/fluid/`
- Realm/client/user admin:
  - frontend pages usually under `ui/src/pages/...`
  - backend handlers under `src/adapters/web/...`

## Rule of thumb

- If you are reaching for a new top-level directory under `src/` or `ui/src/`, pause first.
- Most changes should fit into an existing domain/application/adapter boundary or an existing FSD slice.
