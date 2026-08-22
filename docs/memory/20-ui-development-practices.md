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
- Any non-zero `ring-offset-*` must be paired with `ring-offset-background`.
  Tailwind's `--tw-ring-offset-color` defaults to `#fff`, so a bare offset paints a
  white ring — unnoticeable on a light theme, glaring on this dark one. Guarded
  repo-wide by `shared/ui/focus-ring.test.ts`.
- Note that the shadow tokens in `theme.css` are 1px white-alpha *rings*, not drop
  shadows (`--shadow-sm: 0 0 0 1px rgba(255,255,255,0.14)`). `shadow-sm` is how
  inputs get their border, so stacking it inside another bordered container gives
  a double border — the Fluid renderers pass `shadow-none` for exactly this reason.
- For a settings/detail card use `SectionCard` (`shared/ui/section-card.tsx`), not
  a hand-assembled `Card` + `CardHeader` + inset `div`. `CardContent` is
  deliberately `p-1`; the inset `bg-primary-foreground rounded-2xl p-4` panel is
  what supplies the content padding, so writing the card by hand and forgetting
  the panel leaves the content at a different inset from the title. That is
  exactly the bug the setup page shipped with.
- `SetupPage` is a deliberate exception: it is a centred *hero* card (logo, centred
  title/description, single surface with no inset panel), which is a different
  archetype from a settings card. It composes the `Card` primitives directly and
  pays for its own `CardContent` padding. Do not "fix" it back to `SectionCard`.
  If a second hero card appears, promote that shape rather than adding
  align/media/flush variants to `SectionCard` — the inset panel is the defining
  feature of `SectionCard`, and a `flush` flag would negate it.
- Known duplication: `features/realm/components/RealmSettingsCard.tsx` predates
  `SectionCard` and implements the same pattern, and roughly a dozen settings tabs
  (client, events, user, group, roles) still hand-write the inset panel. Migrate
  them to `SectionCard` when touching those files.

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

## 11. Animation
- JS-driven animation goes through the engine in `shared/lib/animations`
  (`fadeSlideIn`, `fadeSlideOut`, `highlight`, `morphSize`). Do not import `gsap`
  directly in a component — the engine indirection is what makes the library
  swappable.
- Prefer CSS for ambient decoration. A looping background effect is a
  `@keyframes` plus an `@utility` in `app/style/index.css` (see `pulse-glow`),
  which costs no JS and stays on the compositor. Reach for the JS engine only
  when the animation needs to react to component state.
- Animate only `opacity` and `transform` for ambient effects, and disable them
  under `@media (prefers-reduced-motion: reduce)`. For JS-driven cases use
  `prefersReducedMotion()` (`shared/lib/animations/prefersReducedMotion.ts`).
- Decorative layers are `aria-hidden="true"` and `pointer-events-none`, and take
  their colours from CSS custom properties (e.g. `--glow-violet`) rather than
  literals.
- Never gate navigation or other user-visible progress on an animation finishing.
- A JS timeline recreated on an interval must replace and kill its predecessor;
  collecting them in an array cleared only on unmount leaks one per tick.

## 12. React Query keys
- Build keys so a short, partially-specified key is a genuine **prefix** of the
  fully-specified one. React Query matches positionally, so a key ending in
  explicit `undefined` matches nothing longer:
  `['theme-preview', realm, id, undefined, undefined]` never matched the live
  `['theme-preview', realm, id, 'login']`, and eight mutation hooks silently
  failed to refresh the theme preview after publish, rollback, and
  start-draft-from-version.
- Optional trailing segments are trimmed by `withoutTrailingUndefined` in
  `shared/lib/queryKeys.ts`. Inner positions are preserved so a key specifying
  only a later argument cannot collide with one specifying an earlier argument.
- Guarded by `shared/lib/queryKeys.test.ts`, which asserts the prefix property
  directly rather than snapshotting key shapes.
