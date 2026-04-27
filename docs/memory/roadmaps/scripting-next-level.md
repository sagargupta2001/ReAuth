# Feature Roadmap: Next-Level Scripting

## Goal
- Turn scripting into a first-class customization layer for ReAuth flows without turning ReAuth into an unsafe plugin runtime.
- Let operators express advanced flow behavior, context shaping, validation, and orchestration through stable contracts.

## Current state
- `core.ui.scripted` exists with server-side execution, signal handling, publish-time patch validation, and preview tooling.
- `core.logic.scripted` exists with typed `success`/`failure` branching and context mutation.
- `core.logic.subflow` exists with call/return semantics.
- Action Binder can now execute `call_subflow` end-to-end via signal envelopes.
- The script editor is a single-file authoring surface with syntax highlighting and scripted-UI-specific patch tooling.

## Design principles
- Contract-first: every scriptable surface must have a typed request/response shape.
- Safe by default: no raw host access, strict limits, auditable execution.
- Composable: scripts should orchestrate built-in nodes and subflows, not replace the whole engine.
- Observable: publish-time validation and runtime diagnostics must remain first-class.

## Now
- Define a typed host API v1 for scripts.
  - `input`
  - `context`
  - `signal`
  - explicit helpers for `set`, `remove`, `fail`, `continue`, `challenge`
- Define typed result helpers so authors do not hand-roll ad hoc JSON.
- Add backend dry-run endpoints for scripted logic and scripted UI.
- Add editor snippets/templates for common patterns.
- Add richer validation for script return values in the editor before publish.

## Next
- Add read-only secrets access with vault-backed injection.
- Add reusable script libraries.
  - node-local helpers first
  - optional system-level shared helpers later
- Add structured field-error helpers for scripted UI.
- Add typed subflow invocation helpers from scripts.
- Add realm/client/request helper accessors so scripts do not parse raw context blindly.
- Add execution logs/trace surface visible from the builder.

## Later
- Add script test fixtures and saved examples per realm.
- Add versioned reusable script modules in Harbor bundles.
- Add migration tooling for script contract changes.
- Add richer result types for scripted logic beyond binary success/failure where justified by the node contract model.
- Add optional stronger isolation for untrusted scripts.

## Risks / dependencies
- Secrets access must not leak values into theme payloads or browser-visible context.
- A richer host API increases compatibility burden; DTO versioning must precede broad rollout.
- Dry-run execution must remain deterministic and side-effect free.
- Reusable libraries need a migration story before wide adoption.

## Open questions
- Should script libraries be realm-scoped, theme-scoped, or globally packaged first?
- Should subflow invocation from scripts stay declarative or allow dynamic flow-type selection?
- How much request metadata should scripts see by default?
- Do we want a typed mini-DSL for common auth operations before broadening raw JS further?

## Implementation checklist
- [x] Ship `core.ui.scripted`.
- [x] Ship `core.logic.scripted`.
- [x] Ship `core.logic.subflow`.
- [x] Support Action Binder `call_subflow` signals end-to-end.
- [ ] Define typed host API v1.
- [ ] Add backend dry-run/test endpoint for scripts.
- [ ] Add editor-side contract validation for script return payloads.
- [ ] Add snippet/template library for common scripted patterns.
- [ ] Add safe read-only vault secret access.
- [ ] Add reusable script helper libraries.
- [ ] Add builder-visible script execution traces.
- [ ] Define contract/version migration policy for script APIs.
