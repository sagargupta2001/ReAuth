# Feature Roadmap: Fluid Action Binding (Signal/Intent Bridge)

## Goal
- Bind Fluid UI components to server-side flow logic without custom frontends.
- Preserve session security by avoiding raw client JavaScript.
- Keep the action model deterministic and publish-time validated.

## Core concept
- **Signals** are declarative intents emitted by Fluid components.
- The **Bridge** maps signals to backend flow execution.
- **Scripted UI nodes** (or built-in nodes) handle the signal and return an outcome.

## Data model (UI)
- Every interactive Fluid block can define `actions`.
- Each action defines:
  - `trigger`: `on_click` | `on_submit` | `on_load` | `on_change`
  - `signal`: `{ type, node_id, payload_map }`

Example (Fluid blueprint snippet):
```json
{
  "type": "Component",
  "component": "Button",
  "props": {
    "label": "Continue",
    "actions": [
      {
        "trigger": "on_click",
        "signal": {
          "type": "submit_node",
          "node_id": "password",
          "payload_map": {
            "email": "inputs.email",
            "password": "inputs.password"
          }
        }
      }
    ]
  }
}
```

## Bridge behavior (UI -> API)
- UI collects mapped values and posts a structured payload to the backend execute endpoint.
- Payload schema:
```json
{
  "signal": {
    "type": "submit_node",
    "node_id": "password",
    "payload": {
      "email": "user@example.com",
      "password": "***"
    }
  }
}
```

## Backend handling
- The flow executor routes signals to the current node (or referenced `node_id`).
- Scripted UI nodes can:
  - Validate inputs and return field errors.
  - Emit `challenge` with a Fluid patch (UI updates).
  - Transition to the next node.

## Outcomes
- `challenge`: return updated UI context or patch for Fluid.
- `continue`: advance flow to next node.
- `reject`: end flow with a terminal state.

## Publish-time validation
- Verify `signal.node_id` exists in the graph.
- Verify `payload_map` references valid component inputs.
- Validate scripted UI patches against Fluid schema (dry run).

## Action Binder UI (admin experience)
- When a block is selected in Fluid, the right sidebar shows an **Actions** tab.
- UI elements:
  - Trigger dropdown: `On Click`, `On Submit`, `On Load`.
  - Action type selector: `Submit Node`, `Trigger Validation`, `Call Subflow`, `Execute Script`.
  - Data mapping table: map `Component Value -> Script Input`.
- Visual language:
  - Dark mode, technical aesthetic.
  - Small connector/plug iconography to imply linkage.

## Next actions
- Document signal/action binding payloads with examples in the public docs.

## Implementation checklist
- [x] Define backend signal payload schema.
- [x] Extend execute endpoints to accept signal envelopes.
- [x] Add publish-time validation for signal bindings.
- [x] Validate `payload_map` shape and allowed path roots.
- [x] Emit signal envelope from Fluid runtime (on_submit + on_click).
- [x] Warn in UI when `payload_map` resolves to undefined values.
- [x] Add Fluid editor Action Binder UI.
- [x] Inline editor validation for payload_map paths.
- [x] Add mapping helpers for inputs/context paths.
- [x] Add payload_map validation against actual component inputs.
- [x] Add scripted UI node execution for custom signal handlers.
- [x] Add node_id picker suggestions from flow graphs.
- [x] Add scripted UI authoring UX (inline editor + file load).
- [x] Add script sandbox limits for scripted UI execution.
- [x] Default node_id picker to realm browser flow when no flow draft is supplied.
- [x] Update picker helper text to show flow source.
- [x] Add optional theme -> flow binding for Action Binder suggestions.
- [x] Add manual flow selector in Theme Builder when no binding is present.
- [x] Add publish-time dry-run validation for scripted UI patches.
- [x] Add theme settings control to change/clear flow binding.
- [x] Add integration tests for scripted UI dry-run validation.
- [x] Add docs/examples for scripted UI patch payloads.
- [x] Add script editor dialog with syntax highlighting.
- [x] Add template-key suggestions in script editor.
- [x] Add UI patch preview in script editor.
- [x] Add schema-aware autocomplete + validation for ui_patch JSON.
- [x] Add template-key insertion menu with search + keyboard nav.
- [x] Render ui_patch preview with active theme tokens when available.
- [x] Add schema-aware property key autocomplete for ui_patch JSON.
- [x] Add template-key toolbar quick actions + current-page helper.
- [x] Add inline schema docs/help for ui_patch JSON (hover hints).
- [x] Add "Validate patch" button for ui_patch JSON.
- [x] Add diff view for ui_patch preview.
- [x] Add end-to-end coverage for Action Binder -> signal execution (UI + API).
- [x] Surface publish-time action binding failures with node jump links in builder.
