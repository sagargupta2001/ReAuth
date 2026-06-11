# Spec: User Profile Phone Numbers

> Distilled from: admin user-management request on 2026-06-10
> Status: Implemented

---

## User Story

As a realm admin, I want to manage a user's profile names and phone numbers alongside email addresses so that contact identity data is consistent, editable, and auditable from the user detail page.

---

## Actors

| Actor | Role in this feature |
|-------|----------------------|
| Realm Admin | Updates profile names, adds/removes phone numbers, sets primary phone numbers, and marks phone numbers verified or unverified |
| End User | Has first name, last name, emails, and phone numbers represented on their account |
| OIDC Client | No direct impact in this slice |
| Operator | No direct impact in this slice |

---

## Business Rules

1. A user profile has optional `first_name` and `last_name` fields in addition to `username`.
2. `first_name` and `last_name` are editable from the profile section and may be cleared.
3. A user may have zero or more phone numbers.
4. Phone numbers are realm-unique after normalization.
5. A phone number has `is_primary` and `is_verified` flags matching the email-address model.
6. At most one phone number per user may be primary; setting a new primary demotes the previous primary.
7. Admins may add a phone number as primary and/or verified.
8. Admins may mark an existing phone number verified or unverified.
9. Admins may remove non-primary phone numbers. Removing the only phone number is allowed. Removing a primary phone number while other phone numbers exist is rejected until another number is promoted.
10. User detail API responses include the primary phone number and the full phone-number list, mirroring email response shape.
11. Each user-detail tab shows a right-side summary panel with copyable user ID, primary email, username, user creation date, and profile last-updated date.

**Edge cases:**
- Duplicate phone numbers in the same realm are rejected even across different users.
- Phone number input is trimmed and normalized before storage.
- Empty phone number input is rejected.
- The profile summary displays an empty placeholder when the user has no primary email or updated timestamp.

---

## Domain Changes

### New Entities

```text
UserPhoneNumber
  - id: Uuid
  - user_id: Uuid
  - realm_id: Uuid
  - phone_number: String
  - phone_number_normalized: String
  - is_primary: bool
  - is_verified: bool
  - created_at: DateTime<Utc>
  - updated_at: DateTime<Utc>
```

### Modified Entities

```text
User
  + first_name: Option<String>
  + last_name: Option<String>
  + updated_at: Option<DateTime<Utc>>
```

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/user.rs` | Add profile name fields and updated timestamp |
| `domain/user_phone_number.rs` | Add phone-number domain entity |
| `ports/user_phone_number_repository.rs` | Add phone-number repository contract |
| `application/user_service.rs` | Update profile mutation to include names |
| `application/user_phone_number_service.rs` | Add phone-number management service mirroring email behavior |
| `adapters/web/user_handler.rs` | Add phone-number sub-resource endpoints and extend user responses |
| `adapters/persistence/...` | Add SQLite repository and migration |
| `ui/src/features/user/...` | Add phone-number hooks/components, editable name fields, and user summary panel |

---

## Persistence Changes

### New Migration(s)

```text
20260610000000_add_user_profile_names_and_phone_numbers.sql
```

### Data Notes

- `users.first_name`, `users.last_name`, and `users.updated_at` are nullable/backward-compatible additions.
- `user_phone_numbers` is realm-scoped, cascades on user/realm deletion, and has a unique `(realm_id, phone_number_normalized)` constraint.
- SQLite triggers demote existing primary phone numbers when another phone number is inserted or updated as primary.

---

## API Changes

### New HTTP Endpoints

```text
GET /api/realms/{realm}/users/{id}/phone-numbers
  Response: UserPhoneNumber[]
  Auth:     user:write

POST /api/realms/{realm}/users/{id}/phone-numbers
  Request:  { phone_number: string, is_primary?: boolean, is_verified?: boolean }
  Response: UserPhoneNumber
  Auth:     user:write

DELETE /api/realms/{realm}/users/{id}/phone-numbers/{phone_number_id}
  Response: { status: "removed" }
  Auth:     user:write

PUT /api/realms/{realm}/users/{id}/phone-numbers/{phone_number_id}/primary
  Response: { status: "updated" }
  Auth:     user:write

PATCH /api/realms/{realm}/users/{id}/phone-numbers/{phone_number_id}/verified
  Request:  { is_verified: boolean }
  Response: { status: "updated" }
  Auth:     user:write
```

### Modified Endpoints

```text
PUT /api/realms/{realm}/users/{id}
  Added to request:  { first_name?: string | null, last_name?: string | null }
  Changed response:  includes first_name, last_name, updated_at

GET /api/realms/{realm}/users/{id}
  Changed response:  adds phone_number?: string | null, phone_numbers: UserPhoneNumber[]
```

---

## Flow / Auth Impact

- Flow types affected: none
- New nodes: none
- Existing nodes modified: none
- Async pause/resume impact: none
- Theme or Fluid page impact: none

---

## Availability / Admin UX

- System/operator prerequisites: none
- Realm policy: none in this slice
- Flow composition: none
- Builder behavior: none
- Simple mode UX: user profile tab gains editable names, a phone-number section, and a right-side summary panel
- Advanced mode UX: none

---

## Test Scenarios

1. **Profile name update**
   - Given: a user exists
   - When: an admin updates username, first name, and last name
   - Then: the user response includes the changed profile fields and `updated_at` is refreshed.

2. **Phone number lifecycle**
   - Given: a user exists
   - When: an admin adds two phone numbers, promotes the second, and marks it verified
   - Then: only the second number is primary and its verification status is true.

3. **Duplicate phone validation**
   - Given: a phone number exists for a user in a realm
   - When: an admin adds the same normalized number to another user in the same realm
   - Then: the API returns a field validation error.

4. **Primary removal guard**
   - Given: a user has two phone numbers and one is primary
   - When: an admin attempts to remove the primary number
   - Then: the API rejects the removal until another phone number is primary.

5. **Profile summary panel**
   - Given: a user detail tab renders
   - When: profile data loads
   - Then: the right-side panel shows copyable user ID, primary email, username, user since, and profile last updated.

---

## Out of Scope

- SMS delivery, phone verification challenges, or login by phone number.
- Phone-number formatting by country/region.
- User self-service profile editing.
- OIDC claim emission for first name, last name, or phone number.

---

## Open Questions

- [ ] Should phone numbers become login identifiers in a future auth-flow slice?
- [ ] Should OIDC profile claims expose first name and last name after this admin model exists?
