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
| Columns | `columns` | `Box` | Layout | yes | Element, Auto Layout, Appearance, Size, Placement, Spacing |
| Heading | `heading` | `Text` | Text | no | Element, Content, Size, Placement, Typography, Spacing |
| Checkbox | `checkbox` | `Component` / `Checkbox` | Form Elements | no | Element, Checkbox, Size, Placement, Typography, Spacing |
| Radio Group | `radio-group` | `Component` / `RadioGroup` | Form Elements | no | Element, Options, Size, Placement, Spacing |
| Select | `select` | `Component` / `Select` | Form Elements | no | Element, Options, Select, Size, Placement, Spacing |
| Legal Text | `legal-text` | `Component` / `LegalText` | Text | no | Element, Legal Copy, Size, Placement, Typography, Spacing |

Adding a block: an id in `FluidBlockId`, a definition in `FLUID_BLOCKS`, a
preview in `BLOCK_PREVIEWS` (compile-enforced), and a render target in
`RENDER_TARGETS`. A block reusing an existing target — Columns is a `Box`
preset — needs nothing else. A new target needs one `COMPONENT_RENDERERS`
entry in the shared walker, not a branch per renderer.

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
| Color | `style.typography.color` | `color` |
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
| Label Size | `style.parts.label.typography.size` | `text` |
| Label Weight | `style.parts.label.typography.weight` | `text` |
| Label Color | `style.parts.label.typography.color` | `color` |
| Label Spacing | `style.parts.label.spacing.margin_bottom` | `number` |

### Field

Applies to: Input.

| Field | Writes | Control |
|---|---|---|
| Background | `style.parts.field.fill.color` | `color` |
| Border Color | `style.parts.field.stroke.color` | `color` |
| Border Width | `style.parts.field.stroke.width` | `number` |
| Corner Radius | `style.parts.field.corners.radius` | `number` |
| Inner Padding | `style.parts.field.spacing.padding` | `number` |

### Checkbox

Applies to: Checkbox.

| Field | Writes | Control |
|---|---|---|
| Label | `props.label` | `text` |
| Name | `props.name` | `text` |
| Checked by default | `props.checked` | `select` |

### Options

Applies to: RadioGroup, Select.

| Field | Writes | Control |
|---|---|---|
| Name | `props.name` | `text` |
| Options | `props.options` | `textarea` |
| Default value | `props.value` | `text` |

### Select

Applies to: Select.

| Field | Writes | Control |
|---|---|---|
| Placeholder | `props.placeholder` | `text` |

### Legal Copy

Applies to: LegalText.

| Field | Writes | Control |
|---|---|---|
| Text | `props.text` | `textarea` |

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
| Background | `style.fill.color` | `color` |
| Border Color | `style.stroke.color` | `color` |
| Border Width | `style.stroke.width` | `number` |
| Corner Radius | `style.corners.radius` | `number` |

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
| Text alignment | `style.typography.align` | `select` |

### Typography

Applies to: Text, Button, Link, Checkbox, LegalText.

| Field | Writes | Control |
|---|---|---|
| Font Size | `style.typography.size` | `text` |
| Font Weight | `style.typography.weight` | `text` |
| Color | `style.typography.color` | `color` |

### Spacing

Applies to: every block.

| Field | Writes | Control |
|---|---|---|
| Padding | `style.spacing.padding` | `number` |
| Margin Top | `style.spacing.margin_top` | `number` |
| Margin Bottom | `style.spacing.margin_bottom` | `number` |

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

There is one tree walker, `lib/renderFluidNode.tsx`. `FluidCanvas` (builder
preview) and `FluidLoginScreen` (runtime) drive it with a `FluidHost` each,
supplying only what genuinely differs: wrapping, visibility, and the
interactive leaves. Structure and styling are shared, so a block cannot
render in one and not the other — which is how `ProviderButtons` once
shipped broken in the builder.

| Block | Handled by |
|---|---|
| Box | walker node-type branch |
| Text | walker node-type branch |
| Input Field | `componentRegistry` expansion |
| Button | `COMPONENT_RENDERERS` entry |
| Sign-in Providers | `COMPONENT_RENDERERS` entry |
| Divider | `COMPONENT_RENDERERS` entry |
| Link | `COMPONENT_RENDERERS` entry |
| Image | walker node-type branch |
| Columns | walker node-type branch |
| Heading | walker node-type branch |
| Checkbox | `COMPONENT_RENDERERS` entry |
| Radio Group | `COMPONENT_RENDERERS` entry |
| Select | `COMPONENT_RENDERERS` entry |
| Legal Text | `COMPONENT_RENDERERS` entry |

Host responsibilities, and nothing else:

| Host method | FluidCanvas | FluidLoginScreen |
|---|---|---|
| `isVisible` | always true, so hidden nodes stay editable | gates on `visible` / `visible_if` |
| `wrap` | selection ring + click target | plain sized wrapper |
| `renderText` | shows the binding, dimmed | resolves it against auth context |
| `renderInput` | inert or placeholder | `FormField`-wired input |
| `renderButton` | disabled | actions, OAuth, resend, submit |
| `renderProviders` | preview, or a placeholder when none | live buttons, or hides the block |
| `linkProps` | `preventDefault` | — (navigates) |

## 5. Derived gaps

### Controls whose value no renderer reads

None. Every inspector field writes a prop a renderer consumes.

### Props a renderer reads with no inspector control

Capability that already renders but can only be reached by hand-editing a
blueprint. Annotated entries are deliberate; the rest are candidates for a new
control.

| Prop | Status |
|---|---|
| `props.align` | Legacy spelling of `style.typography.align`. |
| `props.background` | Legacy spelling of `style.fill.color`. |
| `props.border_color` | Legacy spelling of `style.stroke.color`. |
| `props.border_width` | Legacy spelling of `style.stroke.width`. |
| `props.color` | Legacy spelling of `style.typography.color`. |
| `props.field_background` | Legacy spelling of `style.parts.field.fill.color`. |
| `props.field_border_color` | Legacy spelling of `style.parts.field.stroke.color`. |
| `props.field_border_width` | Legacy spelling of `style.parts.field.stroke.width`. |
| `props.field_padding` | Legacy spelling of `style.parts.field.spacing.padding`. |
| `props.field_radius` | Legacy spelling of `style.parts.field.corners.radius`. |
| `props.font_size` | Legacy spelling of `style.typography.size`. |
| `props.font_weight` | Legacy spelling of `style.typography.weight`. |
| `props.height` | Legacy pre-`size` fallback, still read for old blueprints. |
| `props.height_value` | Legacy pre-`size` fallback, still read for old blueprints. |
| `props.label_color` | Legacy spelling of `style.parts.label.typography.color`. |
| `props.label_size` | Legacy spelling of `style.parts.label.typography.size`. |
| `props.label_spacing` | Legacy spelling of `style.parts.label.spacing.margin_bottom`. |
| `props.label_weight` | Legacy spelling of `style.parts.label.typography.weight`. |
| `props.margin_bottom` | Legacy spelling of `style.spacing.margin_bottom`. |
| `props.margin_top` | Legacy spelling of `style.spacing.margin_top`. |
| `props.padding` | Legacy spelling of `style.spacing.padding`. |
| `props.radius` | Legacy spelling of `style.corners.radius`. |
| `props.visible` | Toggled by the Input slots panel, not by a schema field. |
| `props.visible_if` | Authored in the blueprint; binds the block to auth context. |
| `props.width` | Legacy pre-`size` fallback, still read for old blueprints. |
| `props.width_value` | Legacy pre-`size` fallback, still read for old blueprints. |

Heuristic: props are collected by scanning `props.<key>` in the renderers,
`nodeVisuals.ts`, and `componentRegistry.ts`. A prop read through a computed
key would not appear here.
