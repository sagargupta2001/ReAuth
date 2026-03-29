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
- Implement the action schema in Fluid block model.
- Add bridge payload support in the auth execute endpoints.
- Add publish-time validator for signal bindings.
