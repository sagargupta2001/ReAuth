# Feature Roadmap: Omni Search (Command Palette)

## Goal
- Deliver a Raycast-style, global command palette that unifies navigation, settings, and entity search for ReAuth with fast, contextual actions.

## Current state (code-aligned)
- [x] Baseline command palette exists (`ui/src/features/Search/components/CommandMenu.tsx`).
- [x] Shadcn `Command` primitives are available via `ui/src/shared/ui/command.tsx`.
- [x] Omni split-pane palette implemented with static + dynamic search and inspector.
- [x] Dynamic search includes users, clients, roles, groups, and flows.
- [x] Observability links (Logs/Traces/Cache) and danger actions are indexed in static search.
- [x] Dangerous actions require confirmation in omni search and offer “Open in Observability”.
- [x] Keyboard navigation now syncs the inspector.
- [x] Split-pane accessibility: skip links, focus rings, and inspector `aria-describedby`.

## Proposed system design (from project notes)
### Unified search architecture
- **Synchronous (static index):** Navigation routes, settings, and inline actions loaded into a client-side JSON index at app start for instant search.
- **Asynchronous (dynamic DB search):** Users/Clients/Roles via `GET /api/realms/{realm}/search?q=...` with 300ms debounce.

### Scroll & highlight mechanism
- Palette navigates to URL hashes: `/realms/master/settings/general#toggle-registration`.
- Each setting section wraps in a container with matching `id`.
- `useHashScrollHighlight` hook detects `window.location.hash`, calls `scrollIntoView({ behavior: 'smooth' })`, and applies a temporary highlight class.

### Actionable palette items
- Static settings entries include `type: 'toggle'` and `apiEndpoint`.
- Clicking the row navigates; clicking the switch toggles via background mutation and optimistic update without closing the palette.

### UI/UX concept
- Split-pane command palette:
  - Left: input + grouped results (Settings, Users, Clients, Actions).
  - Right: inspector panel that renders context for the highlighted item.

## Now (MVP status)
1. **Define the data contracts** (done)
   - Static index schema: `id`, `label`, `group`, `icon`, `href`, `type`, `description`, `apiEndpoint`, `hashTarget`.
   - Dynamic result schema: `type`, `id`, `label`, `subtitle`, `meta` (roles, last_login, status, etc.).
   - Permission gating rules (what should be hidden for non-admins).
2. **Create the split-pane palette layout** (done)
   - New components: `OmniCommandPalette.tsx`, `PaletteInspector.tsx`.
   - 60/40 layout, sticky inspector, subtle divider, larger modal (`max-w-4xl`).
3. **Implement specialized rows** (done)
   - `CommandSettingRow.tsx` with inline `Switch` and stop-propagation on toggle.
   - `CommandEntityRow.tsx` with avatar/logo, stacked primary/secondary text.
4. **Add scroll + highlight behavior** (done)
   - `useHashScrollHighlight` hook + GSAP-driven `animate-target-highlight` class.
   - Update settings pages to wrap targets with stable `id` values.
5. **Wire the static index** (done)
   - Load once at app start; integrate into palette groups.
   - Include settings + navigation + quick actions.
6. **Implement dynamic search API** (done)
   - Backend endpoint: `GET /api/realms/{realm}/search?q=...`.
   - Query users, clients, roles, groups, flows; return capped, relevance-sorted results.
   - Add 300ms debounce + cancellation in the UI.
7. **Accessibility + keyboard UX** (done)
   - `Alt+1/2/3` to jump to the first three visible groups.
   - Inspector updates on keyboard selection.
   - Skip links + focus rings for split-pane regions.
8. **Observability + danger actions** (done)
   - Static items link to Logs/Traces/Cache tabs.
   - Clear logs/traces + flush cache actions prompt confirmation and optionally open the relevant page.
   - Added Traces danger zone and hash targets for Logs/Traces/Cache.

## Next (polish + performance)
1. **Inspector richness** (in progress)
   - User card: avatar, status badge, quick actions, open profile.
   - Setting card: icon, description, breadcrumb, open setting action.
   - Remaining: last login + roles summary.
2. **Keyboard UX + accessibility** (done)
   - Arrow-key focus syncs with inspector.
   - `aria` roles for split-pane content and inspector announcements.
   - Skip links and focus rings.
3. **Result ranking + grouping** (done)
   - Recency boost for recently used items.
   - Sticky “Suggested Actions” group.
4. **Caching + prefetch** (partial)
   - Cache dynamic search results by query (React Query `staleTime`).
   - Prefetch user profile fragments for the inspector (pending).

## Later (advanced capabilities)
- Query plugins to register their own palette items.
- Add workspace-wide deep links for audit/logs, flow builder nodes, observability queries.
- Saved searches and pinned actions.
- Federation into the API (cross-realm super admin search).
- Multi-select batch actions for users/clients (suspend, assign roles).
- RBAC-aware feature flagging of palette items per tenant.
- Replace fuzzy ranking with vector or trigram ranking for large realms.

## Risks / dependencies
- Search endpoint scope: needs RBAC filtering to avoid leaking hidden users/clients/roles.
- Hash targets must remain stable or the highlight feature will break.
- Inline toggles need consistent API contracts for optimistic updates.
- Large realms may require pagination or fuzzy matching in backend search.

## Open questions
- What is the authoritative list of settings + hash targets for the static index?
- Should palette queries be scoped to the active realm only, or cross-realm for super admins?
- How should “last login” be surfaced (new API vs existing field)?
- Do we want fuzzy search (cmdk + custom ranking) or prefix-only matching?
