# Feature Roadmap: Theme Engine

## Goal
- Provide a native, first‑class theming system for ReAuth that controls layout, typography, colors, and UI blocks without external plugins.
- Make every authenticator node renderable via a theme‑defined page that can be edited in‑product.

## Current state
- No theme engine or editor exists.
- Auth flow UI is hard‑coded in the frontend.

## Now
- Define the **Theme Schema** (JSON) with versioning and validation:
  - Visual tokens: colors, fonts, spacing, radii, shadows.
  - Layout templates: page regions + block slots.
  - UI blocks: inputs, buttons, text, social buttons, legal text, dividers.
- Add backend storage model:
  - `themes` table + `theme_versions` (draft/published) in `reauth_primary.db`.
  - Per‑realm theme binding and default theme policy.
- Build a **Theme Resolver**:
  - Select theme by realm, client_id, or request param.
  - Fallback to default theme.
- Implement **Server‑side render contract**:
  - `GET /api/realms/{realm}/theme/resolve` returns merged theme + page definition.
  - Ensure stable DTOs for UI consumption.
- Create a minimal **Theme Preview** view in the admin UI:
  - Render theme tokens + page layout from schema.

## Next
- Build the **Theme Editor UI**:
  - Sidebar property inspector for tokens.
  - Drag/drop layout regions and UI blocks.
  - Live preview bound to flow node page.
- Create **Block Library**:
  - Prebuilt block set for auth (email, password, OTP, social, checkbox, submit).
  - Configurable per block (labels, placeholders, validation hints).
- Add **Draft vs Published** workflow:
  - Preview draft on a per‑realm basis.
  - Promote draft to published with audit record.

## Later
- Multi‑tenant theming (per client_id overrides).
- Theme marketplace import/export (JSON + asset pack).
- Accessibility audits and contrast warnings.

## Risks / dependencies
- Requires clear separation of theme data and flow runtime data.
- Editor must guard against invalid layouts that break auth flows.

## Open questions
- Should theme assets (fonts/images) be stored in DB or filesystem?
- Should a theme be attached to a flow version or a realm globally?
