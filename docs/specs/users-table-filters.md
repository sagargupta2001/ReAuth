# Spec: Users Table Filters

> Distilled from: Users table filter pills implementation / 2026-05-09
> Status: Implemented

---

## User Story

As a realm admin, I want the Users table filter pills to be URL-backed and server-side so that large realms can filter users by email and account dates without losing pagination or shareable table state.

---

## Business Rules

1. The Users table keeps pagination, search, sorting, and filter state in route query params.
2. Supported user filters are:
   - `filter_email`: case-insensitive partial match against `users.email`
   - `filter_created_at`: JSON date range with optional `from` and `to`
   - `filter_last_sign_in_at`: JSON date range with optional `from` and `to`
3. Date range filters use inclusive day semantics. `from=2026-05-01` means `>= 2026-05-01T00:00:00Z`; `to=2026-05-09` means `< 2026-05-10T00:00:00Z`.
4. Empty, malformed, or unknown filter values are ignored rather than failing the list endpoint.
5. Pagination metadata must reflect the filtered result set.

**Edge cases:**
- Users with `NULL` email do not match an email filter.
- Users with `NULL last_sign_in_at` do not match a last-signed-in date filter.
- Search (`q`) and filter pills compose with `AND` semantics.

---

## Module Impact

| Module | Change |
|--------|--------|
| `domain/user.rs` | Add typed user list filter value objects |
| `ports/user_repository.rs` | Accept typed filters alongside `PageRequest` |
| `adapters/web/user_handler.rs` | Deserialize supported `filter_*` query params |
| `adapters/persistence/sqlite_user_repository.rs` | Apply indexed SQL filters to count and list queries |
| `ui/src/features/user/api/useUsers.ts` | Serialize route filters into supported API params |
| `ui/src/features/user/components/UsersTable.tsx` | Keep filter pills URL-backed through the shared table hook |

---

## Persistence Changes

### New Migration(s)

```text
20260509120000_add_user_list_filter_indexes.sql - Add realm/date indexes for users list filters
```

### Data Notes

- No persisted shape changes.
- Indexes are additive and scoped by `realm_id`.

---

## API Changes

### Modified Endpoints

```text
GET /api/realms/{realm}/users?page=1&per_page=10&q=alice&filter_email=example.com&filter_created_at={"from":"2026-05-01","to":"2026-05-09"}&filter_last_sign_in_at={"from":"2026-05-01"}
  Response: 200 paginated PageResponse<User>
  Auth:     protected, user:read
```

---

## Test Scenarios

1. **Email filter**
   - Given: users with different emails in the same realm
   - When: `filter_email=example.com` is supplied
   - Then: only matching email rows are returned and pagination totals match.

2. **Created date filter**
   - Given: users created on different days
   - When: a created date range is supplied
   - Then: only users created inside the inclusive range are returned.

3. **Last signed in filter**
   - Given: users with and without `last_sign_in_at`
   - When: a last-signed-in date range is supplied
   - Then: only users with sign-in timestamps inside the range are returned.

4. **Composed filters**
   - Given: search and multiple filters
   - When: all params are supplied
   - Then: filters compose with `AND` and sorting/pagination still apply.

---

## Out of Scope

- Saved filter presets.
- Additional user attributes beyond email, created date, and last signed in.
- Full-text or fuzzy search.
