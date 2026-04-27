# UI Data Fetching Compliance Roadmap

## Goal
Move all UI data access to the React Query + `apiClient` pattern so cache, retries, auth refresh, and error handling are consistent across ReAuth.

## Current state (brief)
All feature areas use `useQuery`/`useMutation` hooks with `apiClient`, and query keys are standardized via `shared/lib/queryKeys.ts`. Component-level data access now goes through hooks.

## Non‑compliant files (source of truth)
- ✅ Resolved (March 2026): setup, OIDC token exchange, Harbor API, and component‑level actions were migrated to React Query + `apiClient`.

## Implementation checklist
- [x] Shared `apiClient` wrapper with refresh logic is in place.
- [x] Core feature APIs already use React Query hooks (flows, themes, sessions, observability, etc.).
- [x] Add `useSetupStatus` query + `useSetupBootstrap` mutation; remove raw `fetch` from `setupStatus.ts` and `SetupPage.tsx`.
- [x] Replace raw `fetch` in OIDC token exchange with `apiClient.postUrlEncoded`.
- [x] Replace Harbor `fetch` client with `apiClient` and keep React Query hooks for jobs/import/export.
- [x] Replace component-level `apiClient` calls:
  - `OmniCommandPalette.tsx`: use `useWebhookMutations`/`useRollWebhookSecret`/`useDeleteWebhook`.
  - `GroupTreePanel.tsx`: fetch ancestors via query client + `useGroup` fetcher.
- [x] Introduce a `shared/lib/queryKeys.ts` module for consistent query key naming.
- [x] Normalize all `queryKey` usage to the shared `queryKeys` helpers (hooks + invalidations).
- [x] Add lint guidance in UI practices doc: “No `fetch` in components; all data access must go through hooks.”
- [x] Enforce with ESLint: ban `fetch` and `apiClient` imports outside feature API hooks.
- [x] Enforce `queryKeys` usage with ESLint (no inline `queryKey: []` arrays).

## Next
Optionally add a codemod to auto-fix `queryKey: []` to `queryKeys.*` for future migrations.

## Risks / dependencies
- Harbor export downloads need blob handling; ensure React Query mutations return typed results and don’t exhaust memory.
- OIDC token exchange uses `application/x-www-form-urlencoded`; ensure `apiClient` supports this without changing server expectations.

## Open questions
- Do we want a codemod (`npx jscodeshift`) to auto-rewrite inline query keys?
