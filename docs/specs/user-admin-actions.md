# User Admin Actions

Status: Implemented

## Scope

Admins can inspect a user record as JSON from the user detail header and perform high-risk account actions from an Actions menu.

## Business Rules

- Show JSON displays the full admin user response returned by `GET /api/realms/{realm}/users/{id}` and supports copy-to-clipboard.
- Lock user temporarily prevents sign-in until `users.locked_until`; the duration uses the realm `lockout_duration_secs` setting.
- Ban user indefinitely prevents sign-in while `users.banned_at` is set.
- Delete user permanently removes the user.
- Admins cannot lock, ban, or delete their own account through these endpoints.
- Lock and ban revoke all refresh sessions for the target user.

## Permissions

- `user:delete`: required for bulk user deletion.
- `user:lock`: required for `POST /users/{id}/lock`.
- `user:ban`: required for `POST /users/{id}/ban`.
- Fresh setup assigns these permissions to `super_admin`; seeding backfills them onto an existing `super_admin` role when present.

## API

```text
DELETE /api/realms/{realm}/users
  Request:  { user_ids: uuid[] }
  Response: { status: "deleted", count: number }
  Auth:     protected, user:delete

POST /api/realms/{realm}/users/{id}/lock
  Response: UserResponse
  Auth:     protected, user:lock

POST /api/realms/{realm}/users/{id}/ban
  Response: UserResponse
  Auth:     protected, user:ban
```

## Persistence

- `users.locked_until DATETIME`
- `users.banned_at DATETIME`

## Sign-In Behavior

Password, passkey, and OAuth-broker sign-in paths reject users whose `banned_at` is set or whose `locked_until` is in the future.

## Tests

- Admins with `user:lock`, `user:ban`, and `user:delete` can perform the respective actions.
- `user:write` alone cannot lock, ban, or delete users.
- Lock and ban revoke active refresh sessions.
