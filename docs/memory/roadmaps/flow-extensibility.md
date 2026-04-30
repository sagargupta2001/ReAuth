# Feature Roadmap: Flow Extensibility & Custom Nodes

## Goal
- Let operators build any auth or identity flow they can imagine without forking ReAuth.
- Provide a scalable, safe, and observable customization model with strong runtime guarantees.
- Keep node creation ergonomic while preserving SOLID boundaries and long-term maintainability.

## Guiding principles
- Stable contracts: Node execution, UI binding, config schema, and outputs are versioned interfaces.
- Safety first: Built-in nodes and declarative configuration stay constrained, observable, and auditable.
- Composability: Nodes are small building blocks; complex behavior emerges from composition.
- Upgradeability: Custom nodes can be migrated forward without breaking flows.

## Current state
- UI-capable nodes have page binding metadata and publish-time validation.
- Basic node types exist for password, consent, recovery, OTP verification, and condition.
- Subflow runtime primitives already exist in the core engine.

## Now (Foundation)
- Define a **Node Capability Model** shared across backend and UI. Capabilities: `supports_ui`, `ui_surface`, `allowed_page_categories`, `async_pause`, `side_effects`, `requires_secrets`.
- Create a **Node Contract** versioned DTO. Fields: inputs, outputs, config schema, execution contract, UI contract.
- Introduce a **Flow Compatibility Validator** that validates node contracts, page bindings, and output wiring at publish time.
- Add **Flow diagnostics** to the builder that surface publish errors with actionable pointers and node jump links.
- Establish **Node registry versioning** so flows store the runtime node version used at publish.
- Add **Signal/Intent bindings** between Fluid components and backend node actions.
- Define the **Action Binding** spec and Action Binder UI (see `flow-action-binding.md`).

## Next (Built-In Node Growth)
- Add **Node Parameterization** patterns for built-in nodes. Example: password policy knobs, risk thresholds, conditional expressions, rate limits.
- Introduce **Subflow Nodes** with explicit call/return semantics and input/output mapping.

## Later (Marketplace-grade extensibility)
- **Custom Node Packaging** via Harbor bundles with versioned node definitions and UI assets.
- **Node SDK** for external contributors with local test harness, schema validation, and publish checks.
- **Policy-as-data** library that standardizes policy DSL across nodes.

## Next actions
- Define how to migrate node contracts across breaking changes.
- Document reusable composition patterns for built-in nodes + subflows.

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
- Node contract versioning needs a stable migration story.
- Action bindings must never expose raw session credentials to the browser.

## Decisions (aligned with extensibility goals)
- Secrets: **Vault pattern**. UI stores secret key references; real values are injected server-side only.
- UI action wiring: **Signal/Intent bindings**, not raw client JS, to preserve session security.

## Implementation checklist
- [x] Add backend signal payload schema and validation.
- [x] Extend auth execute endpoints for signal envelopes.
- [x] Validate signal bindings and payload_map at publish time.
- [x] Emit signal envelopes from Fluid runtime.
- [x] Warn when payload_map resolves to undefined in UI.
- [x] Build Action Binder UI in the theme editor.
- [x] Inline editor validation for payload_map paths.
- [x] Add mapping helpers for inputs/context paths.
- [x] Add payload_map validation against component bindings.
- [x] Add node_id picker suggestions from flow graphs.
- [x] Define Node Capability Model shared by backend + UI.
- [x] Introduce versioned Node Contract DTO and persist contract versions at publish time.
- [x] Add builder Flow diagnostics (publish errors with node jump links and inline guidance).
- [x] Remove theme -> flow binding after switching Action Binder to flow-agnostic autocomplete.
- [ ] Define node contract migration strategy for breaking changes.
- [x] Introduce subflow nodes with explicit call/return semantics.
- [x] Execute `call_subflow` signals end-to-end from Fluid Action Binder.
- [ ] Document reusable built-in-node/subflow composition patterns.
