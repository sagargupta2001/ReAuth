# Feature Roadmap: Embedded Scripting Runtime

## Goal
- Enable in‑product scripting for flow logic and UI blocks while staying single‑binary.
- Keep the engine swappable (Boa or rquickjs) behind a stable abstraction.

## Current state
- No embedded scripting runtime.
- No host API or sandbox policy defined.

## Now
- Define the **Engine Abstraction** (no engine‑specific types in core code):
  - `ScriptingEngine` trait in `reauth_core` with `compile`, `execute`, `set_timeout`, `set_memory_limit`.
  - `ScriptContext` DTO (input + host functions + output).
  - `ScriptResult` with typed success/failure and structured error details.
- Create **Engine Adapter Crate(s)**:
  - `crates/scripting/boa_adapter` and/or `crates/scripting/quickjs_adapter`.
  - Only adapter crates depend on Boa/rquickjs; core depends on a thin `scripting_core` interface.
- Add **Runtime Selection**:
  - Config flag: `scripting.engine = "boa" | "quickjs"`.
  - Factory selects engine at startup; no call‑sites depend on the concrete engine.
- Implement **Host API surface**:
  - Read‑only: request metadata, realm settings, user profile (if authenticated), flow context.
  - Write‑safe: set context fields, emit validation errors, redirect requests.
  - Explicitly scoped functions (no raw DB access initially).
- Build **Hello World** script execution in flow executor:
  - `onExecute(context, form)` returns `{ action: "continue" | "challenge" | "reject" }`.
  - Log script timing and errors to observability.

## Next
- Add **Sandboxing + Limits**:
  - Per‑script timeouts (e.g., 50–200ms) with hard termination.
  - Memory caps and execution quotas.
- Add **Script Storage Model**:
  - Store per‑node script in `reauth_primary.db` with versioning.
  - Draft/publish workflow (align with theme draft/publish).
- Add **Editor Integration**:
  - Code editor tab per node (Monaco/CodeMirror).
  - Linting hints + safe API docs panel.

## Later
- Allow reusable script modules (per‑realm library).
- Add type‑checked bindings via JSON schema + generated TS typings.
- Provide deterministic execution mode for tests.

## Engine Swap Strategy
- All call‑sites use `ScriptingEngine` + `ScriptContext` interfaces.
- The adapters translate host functions and values into engine‑specific representations.
- No engine‑specific types are exposed outside adapter crates.
- Feature flags gate each engine implementation to keep the binary lean.

## Risks / dependencies
- Sandboxing is critical; a bad script must not block the main runtime.
- Host API must remain minimal and stable to avoid coupling.

## Open questions
- Default engine choice (Boa for safety vs rquickjs for performance).
- Whether to isolate scripts on a dedicated worker thread pool.
