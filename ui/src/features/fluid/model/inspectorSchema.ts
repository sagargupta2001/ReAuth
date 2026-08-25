import {
  CustomInspectorPanel,
  FieldTarget,
  InspectorFieldKind,
  type InspectorSection,
  type SelectOption,
} from '@/features/fluid/model/inspectorFields'

const DIMENSION_MODES: readonly SelectOption[] = [
  { value: 'fill', label: 'Fill' },
  { value: 'hug', label: 'Hug' },
  { value: 'fixed', label: 'Fixed' },
]

/** Flex cross-axis alignment. Distinct from text alignment — see below. */
const CONTENT_ALIGN_OPTIONS: readonly SelectOption[] = [
  { value: 'start', label: 'Start' },
  { value: 'center', label: 'Center' },
  { value: 'end', label: 'End' },
  { value: 'stretch', label: 'Stretch' },
  { value: 'baseline', label: 'Baseline' },
]

const JUSTIFY_OPTIONS: readonly SelectOption[] = [
  { value: 'start', label: 'Start' },
  { value: 'center', label: 'Center' },
  { value: 'end', label: 'End' },
  { value: 'between', label: 'Space between' },
  { value: 'around', label: 'Space around' },
]

/** Text alignment. Distinct from content alignment — see above. */
const TEXT_ALIGN_OPTIONS: readonly SelectOption[] = [
  { value: 'left', label: 'Left' },
  { value: 'center', label: 'Center' },
  { value: 'right', label: 'Right' },
]

const CONTROL_SIZE_OPTIONS: readonly SelectOption[] = [
  { value: 'sm', label: 'Small' },
  { value: 'md', label: 'Medium' },
  { value: 'lg', label: 'Large' },
]

/**
 * Declarative description of the inspector's Properties tab.
 *
 * Sections render in this order, filtered by `appliesTo`. Adding a property is an
 * entry here; adding a node type is a matcher plus its section.
 *
 * Two naming rules this schema exists to enforce:
 *
 * 1. A label must say *which* thing it controls. "Alignment" appeared twice —
 *    once for `layout.align` and once for `props.align` — and "Padding" appeared
 *    twice for `layout.padding` and `props.padding`. They are different
 *    properties on different targets and now read that way.
 * 2. A section must declare the nodes it applies to, so Typography stops
 *    appearing for a Box.
 */
export const INSPECTOR_SECTIONS: readonly InspectorSection[] = [
  {
    id: 'element',
    title: 'Element',
    description: 'The selected block.',
    fields: [
      {
        kind: InspectorFieldKind.Readonly,
        id: 'block-type',
        label: 'Block Type',
        value: (node) =>
          node.type === 'Component' && node.component ? node.component : node.type || 'Node',
      },
    ],
  },
  {
    id: 'text-content',
    title: 'Content',
    description: 'Copy shown in this block.',
    appliesTo: { types: ['Text'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'text',
        label: 'Text',
        target: FieldTarget.Props,
        key: 'text',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'text-path',
        label: 'Bind to context',
        hint: 'Path such as message or error. Overrides the text above at runtime.',
        target: FieldTarget.Props,
        key: 'text_path',
        placeholder: 'e.g. message',
      },
    ],
  },
  {
    id: 'icon',
    title: 'Icon',
    description: 'Glyph and custom SVG.',
    appliesTo: { types: ['Icon'] },
    fields: [
      {
        kind: InspectorFieldKind.Icon,
        id: 'icon-name',
        label: 'Icon Name',
        target: FieldTarget.Props,
        key: 'name',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'icon-size',
        label: 'Size',
        target: FieldTarget.Props,
        key: 'size',
        placeholder: 'e.g. 16',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'icon-color',
        label: 'Color',
        target: FieldTarget.Props,
        key: 'color',
        placeholder: 'e.g. #94a3b8',
      },
      {
        kind: InspectorFieldKind.Textarea,
        id: 'svg-path',
        label: 'Custom SVG',
        hint: 'When an SVG path is provided it overrides the icon name.',
        target: FieldTarget.Props,
        key: 'svg_path',
        placeholder: 'SVG path (d attribute)',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'svg-viewbox',
        label: 'ViewBox',
        target: FieldTarget.Props,
        key: 'svg_viewbox',
        placeholder: 'e.g. 0 0 24 24',
      },
    ],
  },
  {
    id: 'input',
    title: 'Input',
    description: 'Field label, name, and type.',
    appliesTo: { components: ['Input'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'input-label',
        label: 'Label',
        target: FieldTarget.Props,
        key: 'label',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'input-name',
        label: 'Name',
        hint: 'Submitted as this key, and referenced by actions as inputs.<name>.',
        target: FieldTarget.Props,
        key: 'name',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'input-placeholder',
        label: 'Placeholder',
        target: FieldTarget.Props,
        key: 'placeholder',
        placeholder: 'e.g. you@company.com',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'input-type',
        label: 'Input Type',
        target: FieldTarget.Props,
        key: 'input_type',
        placeholder: 'text, email, password',
      },
      {
        kind: InspectorFieldKind.Custom,
        id: 'input-slots',
        panel: CustomInspectorPanel.InputSlots,
      },
    ],
  },
  {
    id: 'input-label-style',
    title: 'Label',
    description: 'Typography for the field label.',
    appliesTo: { components: ['Input'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'label-size',
        label: 'Label Size',
        target: FieldTarget.Props,
        key: 'label_size',
        placeholder: 'e.g. 12px',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'label-weight',
        label: 'Label Weight',
        target: FieldTarget.Props,
        key: 'label_weight',
        placeholder: 'e.g. 600',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'label-color',
        label: 'Label Color',
        target: FieldTarget.Props,
        key: 'label_color',
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'label-spacing',
        label: 'Label Spacing',
        hint: 'Gap between the label and the field, in pixels.',
        target: FieldTarget.Props,
        key: 'label_spacing',
        min: 0,
      },
    ],
  },
  {
    id: 'input-field-style',
    title: 'Field',
    description: 'The box around the input.',
    appliesTo: { components: ['Input'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'field-background',
        label: 'Background',
        target: FieldTarget.Props,
        key: 'field_background',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'field-border-color',
        label: 'Border Color',
        target: FieldTarget.Props,
        key: 'field_border_color',
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'field-border-width',
        label: 'Border Width',
        target: FieldTarget.Props,
        key: 'field_border_width',
        min: 0,
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'field-radius',
        label: 'Corner Radius',
        target: FieldTarget.Props,
        key: 'field_radius',
        min: 0,
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'field-padding',
        label: 'Inner Padding',
        target: FieldTarget.Props,
        key: 'field_padding',
        min: 0,
      },
    ],
  },
  {
    id: 'button',
    title: 'Button',
    description: 'Label and variant.',
    appliesTo: { components: ['Button'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'button-label',
        label: 'Label',
        target: FieldTarget.Props,
        key: 'label',
      },
      {
        kind: InspectorFieldKind.Select,
        id: 'button-variant',
        label: 'Variant',
        target: FieldTarget.Props,
        key: 'variant',
        fallback: 'primary',
        options: [
          { value: 'primary', label: 'Primary' },
          { value: 'secondary', label: 'Secondary' },
          { value: 'outline', label: 'Outline' },
        ],
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'button-intent',
        label: 'Intent',
        hint: 'Submitted as the decision value, e.g. allow, deny, retry, resend.',
        target: FieldTarget.Props,
        key: 'intent',
      },
    ],
  },
  {
    id: 'link',
    title: 'Link',
    description: 'Label and destination.',
    appliesTo: { components: ['Link'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'link-label',
        label: 'Label',
        target: FieldTarget.Props,
        key: 'label',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'link-href',
        label: 'Href',
        target: FieldTarget.Props,
        key: 'href',
        placeholder: '/forgot-password',
      },
      {
        kind: InspectorFieldKind.Select,
        id: 'link-target',
        label: 'Opens In',
        target: FieldTarget.Props,
        key: 'target',
        fallback: '_self',
        options: [
          { value: '_self', label: 'Same tab' },
          { value: '_blank', label: 'New tab' },
        ],
      },
    ],
  },
  {
    id: 'image',
    title: 'Image',
    description: 'Asset and alternative text.',
    appliesTo: { types: ['Image'] },
    fields: [
      {
        kind: InspectorFieldKind.Asset,
        id: 'asset-id',
        label: 'Asset',
        target: FieldTarget.Props,
        key: 'asset_id',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'alt',
        label: 'Alt Text',
        target: FieldTarget.Props,
        key: 'alt',
      },
    ],
  },
  {
    id: 'box-layout',
    title: 'Auto Layout',
    description: 'How this box arranges its children.',
    appliesTo: { types: ['Box'] },
    fields: [
      {
        kind: InspectorFieldKind.Select,
        id: 'direction',
        label: 'Direction',
        target: FieldTarget.Layout,
        key: 'direction',
        fallback: 'column',
        options: [
          { value: 'column', label: 'Column' },
          { value: 'row', label: 'Row' },
        ],
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'gap',
        label: 'Gap',
        hint: 'Space between children, in pixels.',
        target: FieldTarget.Layout,
        key: 'gap',
        min: 0,
      },
      {
        kind: InspectorFieldKind.Select,
        id: 'content-align',
        label: 'Align children',
        hint: 'Across the layout direction. Not the same as text alignment.',
        target: FieldTarget.Layout,
        key: 'align',
        fallback: 'stretch',
        options: CONTENT_ALIGN_OPTIONS,
      },
      {
        kind: InspectorFieldKind.Select,
        id: 'content-justify',
        label: 'Distribute children',
        hint: 'Along the layout direction.',
        target: FieldTarget.Layout,
        key: 'justify',
        fallback: 'start',
        options: JUSTIFY_OPTIONS,
      },
      {
        kind: InspectorFieldKind.PaddingBox,
        id: 'layout-padding',
        label: 'Inner padding',
        hint: 'Inside this box, around its children. Top, right, bottom, left.',
        key: 'padding',
      },
    ],
  },
  {
    id: 'box-appearance',
    title: 'Appearance',
    description: 'Surface and outline of this box.',
    // Every prop here was already read by the `Box` case in both renderers and
    // reachable only by hand-editing the blueprint. The capability matrix in
    // `docs/memory/22-fluid-capability-matrix.md` is what surfaced the gap.
    appliesTo: { types: ['Box'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'box-background',
        label: 'Background',
        hint: 'A colour, or a token reference like var(--card).',
        target: FieldTarget.Props,
        key: 'background',
        placeholder: 'e.g. #0f172a or var(--card)',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'box-border-color',
        label: 'Border Color',
        hint: 'Setting either border field draws the outline.',
        target: FieldTarget.Props,
        key: 'border_color',
        placeholder: 'e.g. #1e293b or var(--border)',
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'box-border-width',
        label: 'Border Width',
        target: FieldTarget.Props,
        key: 'border_width',
        min: 0,
        placeholder: '0',
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'box-radius',
        label: 'Corner Radius',
        hint: 'Pixels. Overrides the theme radius for this box.',
        target: FieldTarget.Props,
        key: 'radius',
        min: 0,
        placeholder: '0',
      },
    ],
  },
  {
    id: 'size',
    title: 'Size',
    description: 'How this block occupies space.',
    fields: [
      {
        kind: InspectorFieldKind.Dimension,
        id: 'width',
        label: 'Width',
        axis: 'width',
        fallback: 'fill',
        options: DIMENSION_MODES,
        valueLabel: 'Custom Width',
      },
      {
        kind: InspectorFieldKind.Dimension,
        id: 'height',
        label: 'Height',
        axis: 'height',
        fallback: 'hug',
        options: DIMENSION_MODES,
        valueLabel: 'Custom Height',
      },
      {
        kind: InspectorFieldKind.Select,
        id: 'control-size',
        label: 'Control Size',
        hint: 'Preset height and text size for inputs and buttons.',
        target: FieldTarget.Props,
        key: 'size',
        fallback: 'md',
        options: CONTROL_SIZE_OPTIONS,
      },
    ],
  },
  {
    id: 'placement',
    title: 'Placement',
    description: 'Where this block sits on the page.',
    fields: [
      {
        kind: InspectorFieldKind.Select,
        id: 'slot',
        label: 'Slot',
        hint: 'Split-screen shells render the brand slot beside the form.',
        target: FieldTarget.Props,
        key: 'slot',
        fallback: 'form',
        options: [
          { value: 'form', label: 'Form' },
          { value: 'brand', label: 'Brand' },
        ],
      },
      {
        kind: InspectorFieldKind.Select,
        id: 'text-align',
        label: 'Text alignment',
        hint: 'Aligns this block’s own text. Not the same as aligning children.',
        target: FieldTarget.Props,
        key: 'align',
        fallback: 'left',
        options: TEXT_ALIGN_OPTIONS,
      },
    ],
  },
  {
    id: 'typography',
    title: 'Typography',
    description: 'Font overrides for this block.',
    // Text-bearing nodes only: it made no sense on a Box or an Image.
    appliesTo: { types: ['Text'], components: ['Button', 'Link'] },
    fields: [
      {
        kind: InspectorFieldKind.Text,
        id: 'font-size',
        label: 'Font Size',
        target: FieldTarget.Props,
        key: 'font_size',
        placeholder: 'e.g. 16px',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'font-weight',
        label: 'Font Weight',
        target: FieldTarget.Props,
        key: 'font_weight',
        placeholder: 'e.g. 600 or bold',
      },
      {
        kind: InspectorFieldKind.Text,
        id: 'font-color',
        label: 'Color',
        target: FieldTarget.Props,
        key: 'color',
      },
    ],
  },
  {
    id: 'spacing',
    title: 'Spacing',
    description: 'Space around this block.',
    fields: [
      {
        kind: InspectorFieldKind.Number,
        id: 'outer-padding',
        label: 'Padding',
        hint: 'Around this block itself. A Box also has its own inner padding.',
        target: FieldTarget.Props,
        key: 'padding',
        min: 0,
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'margin-top',
        label: 'Margin Top',
        target: FieldTarget.Props,
        key: 'margin_top',
        min: 0,
      },
      {
        kind: InspectorFieldKind.Number,
        id: 'margin-bottom',
        label: 'Margin Bottom',
        target: FieldTarget.Props,
        key: 'margin_bottom',
        min: 0,
      },
    ],
  },
]
