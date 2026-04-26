# Spec: [Feature Name]

> Distilled from: [source doc / discussion / PRD section / date]
> Status: Draft | Ready | Implemented

---

## User Story

As a [realm admin / end user / OIDC client / operator], I want to [do something] so that [outcome].

---

## Actors

| Actor | Role in this feature |
|-------|---------------------|
| Realm Admin | [what the admin configures or controls] |
| End User | [what the user experiences] |
| OIDC Client | [if applicable] |
| Operator | [if applicable] |

---

## Business Rules

Numbered list. Each rule should be specific and independently testable.

1. [Rule 1]
2. [Rule 2]
3. [Rule 3]

**Edge cases:**
- [Edge case 1]
- [Edge case 2]

---

## Domain Changes

### New Entities (if any)

```text
EntityName
  - field: type — description
  - field: type — description
```

### Modified Entities (if any)

```text
ExistingEntity
  + new_field: type — description
  ~ modified_field: type — what changes and why
```

### New Value Objects (if any)

```text
ValueObjectName — description
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/...` | [none / what changes] |
| `application/...` | [none / what changes] |
| `adapters/web/...` | [none / what changes] |
| `adapters/persistence/...` | [none / what changes] |
| `ui/src/features/...` | [none / what changes] |

---

## Persistence Changes

### New Migration(s)

```text
YYYYMMDDHHMMSS_description.sql — [purpose]
```

### Data Notes

- [backfill / compatibility / defaulting note]
- [index / constraint / realm scoping note]

---

## API Changes

### New HTTP Endpoints

```text
POST /api/realms/{realm}/[path]
  Request:  { field: type, ... }
  Response: { field: type, ... }
  Auth:     [public | auth required | permission required]
```

### Modified Endpoints (if any)

```text
PATCH /api/realms/{realm}/[path]
  Added to request:  { new_field: type }
  Changed response:  { field: type }
```

---

## Flow / Auth Impact

Use this section when the feature touches login, registration, recovery, OIDC, scripted nodes, or flow builder behavior.

- Flow types affected: [browser / registration / reset / direct / none]
- New nodes: [if any]
- Existing nodes modified: [if any]
- Async pause/resume impact: [none / describe]
- Theme or Fluid page impact: [none / describe]

---

## Test Scenarios

Scenarios that must pass before the feature is complete:

1. **Happy path**
   - Given: [state]
   - When: [action]
   - Then: [result]

2. **Validation failure**
   - Given: [state]
   - When: [invalid action]
   - Then: [result]

3. **Business rule edge case**
   - Given: [state]
   - When: [action]
   - Then: [result]

4. **Error handling**
   - Given: [state]
   - When: [dependency or state failure]
   - Then: [result]

---

## Out of Scope

List nearby but intentionally excluded work:

- [Thing 1]
- [Thing 2]

---

## Open Questions

- [ ] [Question]
- [ ] [Decision pending]
