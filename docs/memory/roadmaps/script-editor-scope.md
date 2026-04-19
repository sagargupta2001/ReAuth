# Script Editor Scope

## Goal
- Define the exact product boundary for custom scripting in ReAuth today.
- Make it clear what operators can build now, what is intentionally out of scope, and what runtime guarantees exist.

## Script surfaces
- `core.ui.scripted`
  - Purpose: handle UI-originated signals on the server and decide whether to re-render, continue, or fail.
  - Inputs: `input`, `context`, `signal`.
  - Return shape:
    - `outcome: "challenge" | "continue" | "reject"`
    - `output?: "success" | "failure"`
    - `context?: object`
  - Special capability:
    - may include `context.ui_patch` to patch the bound Fluid page at challenge time.
- `core.logic.scripted`
  - Purpose: run programmable server-side branching and context mutation inside a flow.
  - Inputs: `input`, `context`, `signal`.
  - Return shape:
    - `output?: "success" | "failure"`
    - `context?: object`
    - `remove_keys?: string[]`

## What users can build today
- Custom server-side validation beyond built-in password or OTP checks.
- Dynamic error messages and conditional UI updates on a Fluid page.
- Progressive profiling and data normalization into session context.
- Realm- or client-specific branching using values already present in flow context.
- Custom decision nodes that route to `success` or `failure`.
- Multi-step experiences by combining:
  - `core.ui.scripted`
  - `core.logic.scripted`
  - `core.logic.subflow`
  - Action Binder signals

## What the editor supports today
- One script per node configuration.
- JavaScript editing in a large dialog.
- Syntax highlighting.
- Load-from-file for script contents.
- Full-screen-style workspace for both scripted UI and scripted logic.
- For `core.ui.scripted` only:
  - left pane script editor
  - right pane `ui_patch` JSON editor
  - patch validation
  - patch diff view
  - rendered preview using the active theme when available
  - template-key helpers and insertion shortcuts

## Runtime guarantees today
- Scripts run server-side only.
- Scripts receive JSON inputs only: `input`, `context`, `signal`.
- Publish-time validation exists for:
  - scripted logic result shape
  - scripted UI patch shape
  - signal bindings
- Sandbox limits exist:
  - logic scripts: 50ms timeout
  - UI scripts: 200ms timeout
  - loop iteration limit
  - recursion limit
  - stack limit
- Output is JSON-only and must match the node contract.

## Hard limits today
- No outbound network access from scripts.
- No direct database access.
- No filesystem access.
- No package imports.
- No secret store access yet.
- No async waiting inside scripts.
- No arbitrary new output handles beyond the node contract.
- No shared script libraries yet.
- No inline debugger or step-through execution.
- No backend dry-run endpoint for authoring-time execution against live flow state.

## Product boundary
- The script editor is currently a flow customization surface, not a general-purpose plugin platform.
- Operators can customize behavior inside a safe contract.
- Operators cannot yet build arbitrary backend integrations or fully new execution runtimes from the editor alone.

## Recommended use cases
- Custom login gate checks.
- Dynamic enrollment rules.
- Conditional consent or step-up decisions.
- Dynamic field validation and inline UI feedback.
- Context shaping before entering a subflow.
- Per-tenant branching without forking built-in nodes.

## Non-goals for the current editor
- Replacing Harbor packaging.
- Running third-party SDKs.
- Long-running orchestration.
- Full application templating or page generation from code.
- Unbounded custom frontend logic in the browser.

## Exit criteria for calling the editor “mature”
- Typed host API documented and stable.
- Script dry-run/test tooling exists.
- Shared helper libraries exist with versioning.
- Secrets access is safe and auditable.
- Result schemas are discoverable from the editor.
- Operators can validate and preview the exact runtime contract before publish.
