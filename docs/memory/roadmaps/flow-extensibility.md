# Feature Roadmap: Flow Extensibility & Custom Nodes

## Goal
- Let operators build any auth or identity flow they can imagine without forking ReAuth.
- Provide a scalable, safe, and observable customization model with strong runtime guarantees.
- Keep node creation ergonomic while preserving SOLID boundaries and long-term maintainability.

## Guiding principles
- Stable contracts: Node execution, UI binding, config schema, and outputs are versioned interfaces.
- Safety first: Scripts and extensions are sandboxed, time-limited, and audited.
- Composability: Nodes are small building blocks; complex behavior emerges from composition.
- Upgradeability: Custom nodes and scripts can be migrated forward without breaking flows.

## Current state
- UI-capable nodes have page binding metadata and publish-time validation.
- Basic node types exist for password, consent, recovery, OTP verification, and condition.
- No user-defined custom nodes or embedded scripting runtime yet.

## Now (Foundation)
- Define a **Node Capability Model** shared across backend and UI. Capabilities: `supports_ui`, `ui_surface`, `allowed_page_categories`, `async_pause`, `side_effects`, `requires_secrets`.
- Create a **Node Contract** versioned DTO. Fields: inputs, outputs, config schema, execution contract, UI contract.
- Introduce a **Flow Compatibility Validator** that validates node contracts, page bindings, output wiring, and scripted UI patches at publish time.
- Add **Flow diagnostics** to the builder that surface publish errors with actionable pointers and node jump links.
- Establish **Node registry versioning** so flows store the runtime node version used at publish.
- Add **Signal/Intent bindings** between Fluid components and backend node actions.
- Define the **Action Binding** spec and Action Binder UI (see `flow-action-binding.md`).

## Next (Custom logic + extensible nodes)
- Embed a **Scripted Logic Node** (rquickjs or Boa via adapter) with safe host API. Host API: flow context, user context, realm policy, read-only secrets, logging, and typed outputs.
- Add a **Scripted UI Node** where server-side scripts validate inputs and return UI state rendered by Fluid pages.
- Add **Node Parameterization** patterns for built-in nodes. Example: password policy knobs, risk thresholds, conditional expressions, rate limits.
- Introduce **Subflow Nodes** with explicit call/return semantics and input/output mapping.

## Later (Marketplace-grade extensibility)
- **Custom Node Packaging** via Harbor bundles with versioned node definitions, scripts, and UI assets.
- **Node SDK** for external contributors with local test harness, schema validation, and publish checks.
- **Policy-as-data** library that standardizes policy DSL across nodes.
- **Advanced runtime isolation** with optional out-of-process sandboxing for untrusted scripts.

## New nodes to explore
- Webhook decision node (async pause/resume).
- Risk scoring node (IP reputation, device fingerprint, geo).
- Step-up authentication node (force MFA or reauth).
- WebAuthn / Passkey node (registration + assertion).
- Magic-link login node (async).
- One-time passcode (SMS/Email) node.
- Device trust node (remember device, step-up on change).
- Account linking node (merge identities across IdPs).
- Identity proofing node (KYC provider integration).
- Token exchange node (OIDC token exchange / reauth).

## New flows to explore
- Passwordless login (magic link + passcode fallback).
- Progressive MFA enrollment (triggered after risk threshold).
- Account recovery with out-of-band approval.
- Admin approval flow for high-privilege access.
- B2B SSO + SCIM onboarding flow.
- Step-up for sensitive actions (billing, export, deletion).
- Device trust and reauthentication cadence.
- Consent update flow on scope changes.

## Risks / dependencies
- Script sandboxing must prevent runaway CPU/memory and unsafe host access.
- Node contract versioning needs a stable migration story.
- UI rendering must remain deterministic even with script-based nodes.
- Action bindings must never expose raw session credentials to the browser.

## Decisions (aligned with extensibility goals)
- Default engine: **Boa-first** for a single-binary Rust runtime; rquickjs remains optional later.
- Script libraries: **Node-local by default**, with an optional `system.*` namespace added later.
- Secrets: **Vault pattern**. UI stores secret key references; real values are injected server-side only.
- Host API (minimal): `context` (realm, client, request, user), `flow` (session + node config), `secrets.get(key)`, `log.*`, `time.now`, `crypto.random`, `assert`.
- Limits: default **50ms** CPU for logic scripts, **200ms** for UI scripts, **8MB** memory, and **1,000,000** instruction budget per call.
- Deterministic UI: **Publish-time dry run** validates scripted UI patches against Fluid schema.
- UI action wiring: **Signal/Intent bindings**, not raw client JS, to preserve session security.
