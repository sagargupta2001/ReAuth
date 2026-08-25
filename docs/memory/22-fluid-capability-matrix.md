# Fluid Capability Matrix

> Generated from the Fluid schemas by
> `ui/src/features/fluid/model/capabilityMatrix.test.ts`. Do not edit by hand —
> run `cd ui && npm run test -- -u` after changing `blockCatalog.ts`,
> `inspectorSchema.ts`, `themeSettingsSchema.ts`, or either renderer.

This is the inventory of what the Fluid builder can currently express: which
blocks exist, what can be styled on each of them, and which global theme
options a page inherits. Use it to see what is already there before adding a
control, and to see the gaps worth closing next. The design rules behind these
lists live in `18-fluid-theme-builder.md` §5.4.

## 1. Blocks

| Block | Catalog id | Renders as | Category | Children | Inspector sections |
|---|---|---|---|---|---|
| Box | `box` | `Box` | Layout | yes | Element, Auto Layout, Appearance, Size, Placement, Spacing |
| Text | `text` | `Text` | Text | no | Element, Content, Size, Placement, Typography, Spacing |
| Input Field | `input` | `Component` / `Input` | Form Elements | no | Element, Input, Label, Field, Size, Placement, Spacing |
| Button | `button` | `Component` / `Button` | Actions | no | Element, Button, Size, Placement, Typography, Spacing |
| Sign-in Providers | `provider-buttons` | `Component` / `ProviderButtons` | Actions | no | Element, Size, Placement, Spacing |
| Divider | `divider` | `Component` / `Divider` | Layout | no | Element, Size, Placement, Spacing |
| Link | `link` | `Component` / `Link` | Text | no | Element, Link, Size, Placement, Typography, Spacing |
| Image | `image` | `Image` | Media | no | Element, Image, Size, Placement, Spacing |

Adding a block: an id in `FluidBlockId`, a definition in `FLUID_BLOCKS`, a
preview in `BLOCK_PREVIEWS` (compile-enforced), and — unless the registry
expands it — a `case` in **both** renderers.

## 2. Per-block styling options

One subsection per inspector section, in the order the inspector renders them.
A field names the exact path it writes, because `props.align` (text) and
`layout.align` (flex cross-axis) are different properties.

### Element

Applies to: every block.

| Field | Writes | Control |
|---|---|---|
| Block Type | — | `readonly` |

### Content

Applies to: Text.

| Field | Writes | Control |
|---|---|---|
| Text | `props.text` | `text` |
| Bind to context | `props.text_path` | `text` |

### Icon

Applies to: Icon (slot-only).

| Field | Writes | Control |
|---|---|---|
| Icon Name | `props.name` | `icon` |
| Size | `props.size` | `text` |
| Color | `props.color` | `text` |
| Custom SVG | `props.svg_path` | `textarea` |
| ViewBox | `props.svg_viewbox` | `text` |

### Input

Applies to: Input.

| Field | Writes | Control |
|---|---|---|
| Label | `props.label` | `text` |
| Name | `props.name` | `text` |
| Placeholder | `props.placeholder` | `text` |
| Input Type | `props.input_type` | `text` |
| input-slots panel | — | `custom` |

### Label

Applies to: Input.

| Field | Writes | Control |
|---|---|---|
| Label Size | `props.label_size` | `text` |
| Label Weight | `props.label_weight` | `text` |
| Label Color | `props.label_color` | `text` |
| Label Spacing | `props.label_spacing` | `number` |

### Field

Applies to: Input.

| Field | Writes | Control |
|---|---|---|
| Background | `props.field_background` | `text` |
| Border Color | `props.field_border_color` | `text` |
| Border Width | `props.field_border_width` | `number` |
| Corner Radius | `props.field_radius` | `number` |
| Inner Padding | `props.field_padding` | `number` |

### Button

Applies to: Button.

| Field | Writes | Control |
|---|---|---|
| Label | `props.label` | `text` |
| Variant | `props.variant` | `select` |
| Intent | `props.intent` | `text` |

### Link

Applies to: Link.

| Field | Writes | Control |
|---|---|---|
| Label | `props.label` | `text` |
| Href | `props.href` | `text` |
| Opens In | `props.target` | `select` |

### Image

Applies to: Image.

| Field | Writes | Control |
|---|---|---|
| Asset | `props.asset_id` | `asset` |
| Alt Text | `props.alt` | `text` |

### Auto Layout

Applies to: Box.

| Field | Writes | Control |
|---|---|---|
| Direction | `layout.direction` | `select` |
| Gap | `layout.gap` | `number` |
| Align children | `layout.align` | `select` |
| Distribute children | `layout.justify` | `select` |
| Inner padding | `layout.padding` | `padding-box` |

### Appearance

Applies to: Box.

| Field | Writes | Control |
|---|---|---|
| Background | `props.background` | `text` |
| Border Color | `props.border_color` | `text` |
| Border Width | `props.border_width` | `number` |
| Corner Radius | `props.radius` | `number` |

### Size

Applies to: every block.

| Field | Writes | Control |
|---|---|---|
| Width | `size.width` + `size.width_value` | `dimension` |
| Height | `size.height` + `size.height_value` | `dimension` |
| Control Size | `props.size` | `select` |

### Placement

Applies to: every block.

| Field | Writes | Control |
|---|---|---|
| Slot | `props.slot` | `select` |
| Text alignment | `props.align` | `select` |

### Typography

Applies to: Text, Button, Link.

| Field | Writes | Control |
|---|---|---|
| Font Size | `props.font_size` | `text` |
| Font Weight | `props.font_weight` | `text` |
| Color | `props.color` | `text` |

### Spacing

Applies to: every block.

| Field | Writes | Control |
|---|---|---|
| Padding | `props.padding` | `number` |
| Margin Top | `props.margin_top` | `number` |
| Margin Bottom | `props.margin_bottom` | `number` |

## 3. Global theme options

Theme-wide tokens, edited in the theme settings panel and inherited by every page.

### Layout

| Field | Writes | Control |
|---|---|---|
| Layout Shell | `layout.shell` | `layout-shell` |

### Assets

| Field | Writes | Control |
|---|---|---|
| Theme Assets | — (theme assets) | `assets` |

### Colors

| Field | Writes | Control |
|---|---|---|
| Primary | `tokens.colors.primary` | `color` |
| Background | `tokens.colors.background` | `color` |
| Text | `tokens.colors.text` | `color` |
| Surface | `tokens.colors.surface` | `color` |
| Contrast | — (read-only report) | `color-contrast` |

### Typography

| Field | Writes | Control |
|---|---|---|
| Font Family | `tokens.typography.font_family` | `text` |
| Base Size | `tokens.typography.base_size` | `number` |

### Effects

| Field | Writes | Control |
|---|---|---|
| Radius | `tokens.radius.base` | `number` |

## 4. Renderer coverage

`FluidCanvas` is the builder preview and `FluidLoginScreen` is what users get.
A block handled by only one of them looks fine in review and is broken in
production, or the reverse — which is exactly how `ProviderButtons` shipped.

| Block | Handled by | FluidCanvas | FluidLoginScreen |
|---|---|---|---|
| Box | inline `case` | yes | yes |
| Text | inline `case` | yes | yes |
| Input Field | `componentRegistry` expansion | yes | yes |
| Button | inline `case` | yes | yes |
| Sign-in Providers | inline `case` | yes | yes |
| Divider | inline `case` | yes | yes |
| Link | inline `case` | yes | yes |
| Image | inline `case` | yes | yes |

## 5. Derived gaps

### Controls whose value no renderer reads

None. Every inspector field writes a prop a renderer consumes.

### Props a renderer reads with no inspector control

Capability that already renders but can only be reached by hand-editing a
blueprint. Annotated entries are deliberate; the rest are candidates for a new
control.

| Prop | Status |
|---|---|
| `props.height` | Legacy pre-`size` fallback, still read for old blueprints. |
| `props.height_value` | Legacy pre-`size` fallback, still read for old blueprints. |
| `props.visible` | Toggled by the Input slots panel, not by a schema field. |
| `props.visible_if` | Authored in the blueprint; binds the block to auth context. |
| `props.width` | Legacy pre-`size` fallback, still read for old blueprints. |
| `props.width_value` | Legacy pre-`size` fallback, still read for old blueprints. |

Heuristic: props are collected by scanning `props.<key>` in the renderers,
`nodeVisuals.ts`, and `componentRegistry.ts`. A prop read through a computed
key would not appear here.
