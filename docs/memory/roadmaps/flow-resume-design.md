# Design: Flow Pause & Resume (Async Steps)

## Why this exists
The current login flow already **resumes UI steps** across page refreshes by keeping
`auth_sessions` alive and using the login-session cookie. That solves “multi-step
UI” flows (e.g., three password steps in a row).

What we still **don’t support** is pausing a flow for **out‑of‑band actions**
like email verification, magic links, or webhook approvals, and then resuming
securely when the user clicks a link or a backend event arrives.

This design adds explicit async pause/resume support while preserving the
existing UI‑resume behavior.

---

## Goals
1. Pause a flow at an async step (email verification, magic link, webhook).
2. Resume with a **one‑time, time‑bound** token.
3. Keep UI state stable and recoverable after refresh.
4. Preserve security: prevent token replay and cross‑realm misuse.

## Non‑Goals (Phase 1)
- Full MFA enrollment flow (Phase 2).
- Rich workflow orchestration beyond basic pause/resume.

---

## Current Behavior (Already in System)
- UI flows are persisted in `auth_sessions`.
- Refreshing the page restores the **current node** and the remaining steps.
- This works for **synchronous UI** nodes but not for async steps.

---

## Proposed Data Model

### 1) `auth_session_actions` (new)
Stores one‑time resume tokens and metadata for async flow steps.

Fields:
- `id` (UUID) — primary key
- `session_id` (UUID, FK → `auth_sessions.id`)
- `realm_id` (UUID, FK → `realms.id`)
- `action_type` (TEXT) — e.g. `email_verify`, `magic_link`, `webhook_approve`
- `token_hash` (TEXT) — hash of the public token
- `payload_json` (TEXT) — optional action metadata (email, client, etc.)
- `resume_node_id` (TEXT) — node to resume at (optional)
- `expires_at` (DATETIME)
- `consumed_at` (DATETIME)
- `created_at` (DATETIME)
- `updated_at` (DATETIME)

### 2) `auth_sessions.context` (existing)
Add:
- `pending_action_id` (UUID)
- `last_ui` (JSON: `{ screen_id, context, updated_at }`)

This lets us re‑render the waiting screen and keep UI state stable after refresh.

---

## Engine Changes

### NodeOutcome (new async variant)
Add a typed async pause:

```
SuspendForAsync {
  action_type: String,
  token: String,
  expires_at: DateTime,
  resume_node_id: Option<String>,
  ui: { screen_id: String, context: Value }
}
```

### FlowExecutor handling
When `SuspendForAsync` is returned:
1. Create `auth_session_actions` row (store hashed token).
2. Save `pending_action_id` and `last_ui` in `auth_sessions.context`.
3. Return a new ExecutionResult:
   - `ExecutionResult::AwaitingAction { screen_id, context }`
   - UI can show “Check your email” screen.

---

## Resume Endpoint

**Endpoint**
`POST /api/realms/{realm}/auth/resume`

Body:
```
{ "token": "<resume-token>" }
```

Flow:
1. Hash token, lookup `auth_session_actions`.
2. Verify:
   - not expired
   - not consumed
   - realm matches
3. Mark `consumed_at`.
4. Update session context (e.g., `context["action_result"] = {...}`).
5. Execute `FlowExecutor::execute(session_id, None)` to continue.

Returns:
- `Challenge` (next UI)
- `Success` (redirect)
- `Failure`
- `AwaitingAction` (if still waiting)

---

## Security Considerations
- Tokens are **one‑time** and **time‑bound**.
- Only hashed tokens stored in DB.
- Realm binding enforced on resume.
- Reuse detection returns error and optionally invalidates the session.

---

## UI Behavior
- If a flow is awaiting an async action, UI shows a “waiting” screen.
- If the page refreshes, the waiting screen is re‑rendered from `last_ui`.
- If the user clicks the verification link, it hits `/auth/resume` which continues the flow.

---

## Migration Plan (Implementation Order)
1. Add `auth_session_actions` migration + repository.
2. Extend `NodeOutcome` + `ExecutionResult`.
3. Implement action creation + resume handler.
4. Store `last_ui` + `pending_action_id` in `auth_sessions`.
5. Add UI waiting screen and `/auth/resume` integration.

---

## Open Questions
- Should resume tokens bind to IP/UA?
- Should action resumption invalidate the entire session on replay?
- Should we enforce a global max outstanding async actions per session?
