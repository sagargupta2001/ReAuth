# Frontend Development Guide

Use this guide together with `docs/memory/20-ui-development-practices.md`.

## Structural rules

- Preserve Feature-Sliced Design:
  - route assembly in `pages`
  - page composition in `widgets`
  - behavior in `features`
  - stable models in `entities`
  - reusable infra/UI in `shared`
- Do not bypass the slice structure by dropping feature behavior into `pages` or `shared`.

## Data access rules

- Do not call `fetch` in components.
- Do not import `apiClient` into components.
- Put network access in typed feature API hooks under `ui/src/features/**/api`.
- Use React Query for data loading and mutations.
- Use shared query key helpers.

## Realm and routing rules

- Preserve realm-aware routing and API scoping.
- Do not hardcode `master` except where explicitly intended for a documented default or first-run path.
- Prefer in-app navigation. Use full page redirects only when auth/session semantics require them.

## UI consistency rules

- Reuse existing shared UI components before adding new primitives.
- Prefer theme tokens and existing patterns instead of local one-off styles.
- For auth/theme/fluid features, keep page-binding and signal-binding conventions intact.

## Forms and validation

- Use `react-hook-form` and schema validation where the feature already follows that pattern.
- Keep request/response typing close to the feature or entity.
- Avoid `any`.

## When modifying an existing feature

1. Start in the feature slice that already owns the behavior.
2. Follow the existing API hook, form, and query-key patterns there.
3. Update page/widget composition only if the user-facing route or page shape changes.

## When creating a new feature

Usually create:

- a feature slice under `ui/src/features/<feature>/`
- typed API hooks in `api/`
- components/forms/schema/model as needed
- page integration only where the route surface changes

Create new shared abstractions only after seeing repeated usage across multiple features.


## React Performance & Rendering

- Avoid creating "God Components" (files exceeding ~500 lines) that mix complex state management, data fetching, and large UI trees.
- Break down massive components into smaller, composable presentational components.
- Push state down to child components or use fine-grained selectors (e.g., Zustand) to prevent excessive top-level re-renders during local state changes (like typing in an input field).
