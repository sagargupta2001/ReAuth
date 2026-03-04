# Feature Roadmap: Embedded Scripting Runtime

## Goal
- Enable in‑product scripting for flow logic and UI blocks while staying single‑binary.
- Keep the engine swappable (Boa or rquickjs) behind a stable abstraction.
- Minimize external dependencies and protect the runtime with strict limits.

## Prerequisite
- Fluid Theme Engine provides stable block IDs and page blueprints used by scripts.

## Current state
- No embedded scripting runtime.
- No host API or sandbox policy defined.

## Now (Phase 2‑A)
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
- Implement **Host API surface** (minimal + safe):
  - Read‑only: request metadata, realm settings, user profile (if authenticated), flow context.
  - UI block helpers: read/write block values, set validation errors, request redirect.
  - Explicitly scoped functions (no raw DB access initially).
- Build **Hello World** script execution in flow executor:
  - `onExecute(context, form)` returns `{ action: "continue" | "challenge" | "reject" }`.
  - Log script timing and errors to observability.

## Next (Phase 2‑B)
- Add **Sandboxing + Limits**:
  - Per‑script timeouts (e.g., 50–200ms) with hard termination.
  - Memory caps and execution quotas.
  - Run on a dedicated thread pool to avoid blocking the main runtime.
- Add **Script Storage Model**:
  - Store per‑node scripts in `reauth_primary.db` with versioning.
  - Draft/publish workflow aligned with Fluid drafts.
- Add **Editor Integration**:
  - Start with a lightweight editor (textarea + minimal highlighting) to avoid heavy deps.
  - Optional enhancement: minimal CodeMirror build if needed.
  - Inline docs panel for host API.

## Later (Phase 2‑C)
- Reusable script modules (per‑realm library).
- Type‑checked bindings via JSON schema + generated TS typings.
- Deterministic execution mode for tests.

## Engine Swap Strategy
- All call‑sites use `ScriptingEngine` + `ScriptContext` interfaces.
- The adapters translate host functions and values into engine‑specific representations.
- No engine‑specific types are exposed outside adapter crates.
- Feature flags gate each engine implementation to keep the binary lean.

## Decisions (best‑practice defaults)
- Default engine: Boa (memory‑safe Rust), with rquickjs optional for performance.
- Script limits are enforced even for trusted admins.
- Scripts are sandboxed and run out‑of‑process if hard termination is needed later.

## Risks / dependencies
- Sandboxing is critical; a bad script must not block the main runtime.
- Host API must remain minimal and stable to avoid coupling.
