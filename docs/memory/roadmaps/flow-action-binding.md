# Feature Roadmap: Fluid Action Binding (Signal/Intent Bridge)

## Goal
- Bind Fluid UI components to server-side flow logic without custom frontends.
- Preserve session security by avoiding raw client JavaScript.
- Keep the action model deterministic and publish-time validated.

## Core concept
- **Signals** are declarative intents emitted by Fluid components.
- The **Bridge** maps signals to backend flow execution.
- Built-in nodes handle the signal and return an outcome.

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
- Built-in nodes can validate inputs, emit a `challenge`, or transition to the next node.

## Outcomes
- `challenge`: return updated UI context or patch for Fluid.
- `continue`: advance flow to next node.
- `reject`: end flow with a terminal state.

## Publish-time validation
- Verify `signal.node_id` exists in the graph.
- Verify `payload_map` references valid component inputs.

## Action Binder UI (admin experience)
- When a block is selected in Fluid, the right sidebar shows an **Actions** tab.
- UI elements:
  - Trigger dropdown: `On Click`, `On Submit`, `On Load`.
  - Action type selector: `Submit Node`, `Trigger Validation`, `Call Subflow`.
  - Data mapping table: map `Component Value -> Node Input`.
- Visual language:
  - Dark mode, technical aesthetic.
  - Small connector/plug iconography to imply linkage.

## Next actions
- Document signal/action binding payloads in public docs.
- Add builder guidance/snippets for when to use `submit_node` vs `call_subflow`.

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
- [x] Add node_id picker suggestions from flow graphs.
- [x] Default node_id picker to realm browser flow when no flow draft is supplied.
- [x] Update picker helper text to show flow source.
- [x] Add end-to-end coverage for Action Binder -> signal execution (UI + API).
- [x] Surface publish-time action binding failures with node jump links in builder.
- [x] Remove theme -> flow binding from Action Binder and theme settings.
- [x] Replace flow-bound node suggestions with local searchable node-id autocomplete.
- [x] Execute `call_subflow` signals end-to-end.
- [ ] Document signal/action binding payloads in public docs.
- [ ] Add authoring guidance for choosing signal types.
