# UI Development Practices (Agent / LLM Guide)

This document defines the baseline UI engineering practices for ReAuth. Follow these rules when adding or changing UI features.

## 1. Data access and state
- Do not call `fetch` inside React components.
- Do not call `apiClient` directly inside React components.
- All network access must go through typed hooks in `ui/src/features/**/api`.
- Use React Query (`useQuery`, `useMutation`) for all data fetching and mutations.
- ESLint enforces “no fetch” and “no apiClient import” outside feature API hooks.
- Prefer a single source of truth for query keys (e.g., `shared/lib/queryKeys.ts`).
- Always invalidate or update relevant queries after mutations.
- Use `apiClient` for authenticated requests. If a request must be `x-www-form-urlencoded` or `multipart/form-data`, add a helper in `apiClient` rather than raw `fetch`.

## 2. Error handling
- Surface errors through existing error boundaries or inline error components; do not swallow errors.
- For auth flows, bubble initialization errors to the error boundary (throw in render path).
- Use `toast` only for transient confirmations or non-blocking failures.

## 3. Loading and UX
- Always show a loading state for queries and mutations.
- Prefer optimistic updates when safe and reversible.
- Avoid inline polling loops; use React Query `refetchInterval` where possible.

## 4. API hook structure
- Each feature should have a dedicated API file and a hook per endpoint.
- Hooks should be named `useXxx` and return typed results.
- Use `enabled` guards for queries that depend on IDs or realm context.
- Keep hook options flexible (pass `options` through to `useQuery`).

## 5. Types and validation
- Define request/response types in the feature or entity model.
- Use schema validation for form inputs where available.
- Avoid `any` or `unknown` in form values unless absolutely necessary.

## 6. Realm awareness
- Always scope API calls by realm when applicable.
- Use the existing realm context hooks and route params, never hardcode `master`.

## 7. Navigation and redirects
- Prefer `useNavigate` and in-app routing.
- When navigation must be a full page load, document why in code comments.

## 8. UI consistency
- Use shared UI components from `ui/src/components` and `ui/src/shared/ui`.
- Avoid custom styling unless necessary; follow theme tokens and Fluid where applicable.

## 9. Testing and linting
- Update or add tests for any new hooks or API behavior.
- Run `npm run lint` and `npm run build` before handing off.

## 10. Checklist (for every UI change)
- [ ] No raw `fetch` in components.
- [ ] New API access is a React Query hook.
- [ ] Query keys are consistent and invalidations are defined.
- [ ] Loading + error states are present.
- [ ] Realm scoping is correct.
- [ ] Types are updated.
- [ ] Lint and build pass.
