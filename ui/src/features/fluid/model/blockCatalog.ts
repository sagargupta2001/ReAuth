import {
  ChevronDown,
  Circle,
  Columns2,
  FileText,
  Fingerprint,
  Heading1,
  Image,
  LayoutTemplate,
  Minus,
  MousePointer2,
  SquareCheck,
  Type,
  type LucideIcon,
} from 'lucide-react'

import type { ThemeNode } from '@/entities/theme/model/types'
import {
  createNodeFromDefinition,
  type ThemeNodeDefinition,
} from '@/features/fluid/lib/nodeUtils'

/**
 * Catalog of blocks a builder can insert into a theme page.
 *
 * Adding a block: add its id to `FluidBlockId`, add the definition to
 * `FLUID_BLOCKS`, and add a preview to `BLOCK_PREVIEWS`. The preview registry is
 * keyed by `FluidBlockId`, so a missing preview is a compile error.
 */
export const FluidBlockId = {
  Box: 'box',
  Text: 'text',
  Input: 'input',
  Button: 'button',
  ProviderButtons: 'provider-buttons',
  Divider: 'divider',
  Link: 'link',
  Image: 'image',
  Columns: 'columns',
  Heading: 'heading',
  Checkbox: 'checkbox',
  RadioGroup: 'radio-group',
  Select: 'select',
  LegalText: 'legal-text',
} as const

export type FluidBlockId = (typeof FluidBlockId)[keyof typeof FluidBlockId]

export const BlockCategory = {
  Layout: 'Layout',
  Text: 'Text',
  FormElements: 'Form Elements',
  Actions: 'Actions',
  Media: 'Media',
} as const

export type BlockCategory = (typeof BlockCategory)[keyof typeof BlockCategory]

/** Display order for categories in the block picker. */
export const BLOCK_CATEGORY_ORDER: readonly BlockCategory[] = [
  BlockCategory.Layout,
  BlockCategory.Text,
  BlockCategory.FormElements,
  BlockCategory.Actions,
  BlockCategory.Media,
]

/**
 * What a *rendered* node is, keyed by how it identifies itself
 * (`component ?? type`).
 *
 * Kept separate from the block catalog because several palette entries can
 * produce the same kind of node — "Columns" is a `Box` with its direction
 * preset to row, and once created it *is* a Box. Deriving the label and the
 * container capability from the block instead meant two presets sharing a
 * render target silently overwrote each other in a `Map`: every Box in the tree
 * would have been relabelled by whichever preset was declared last.
 */
export interface FluidRenderTarget {
  /** `node.component ?? node.type`. */
  key: string
  label: string
  /** Whether this kind of node can contain other blocks. */
  acceptsChildren: boolean
}

export const RENDER_TARGETS: readonly FluidRenderTarget[] = [
  { key: 'Box', label: 'Box', acceptsChildren: true },
  { key: 'Text', label: 'Text', acceptsChildren: false },
  { key: 'Icon', label: 'Icon', acceptsChildren: false },
  { key: 'Image', label: 'Image', acceptsChildren: false },
  { key: 'Input', label: 'Input Field', acceptsChildren: false },
  { key: 'Button', label: 'Button', acceptsChildren: false },
  { key: 'Checkbox', label: 'Checkbox', acceptsChildren: false },
  { key: 'RadioGroup', label: 'Radio Group', acceptsChildren: false },
  { key: 'Select', label: 'Select', acceptsChildren: false },
  { key: 'LegalText', label: 'Legal Text', acceptsChildren: false },
  { key: 'Link', label: 'Link', acceptsChildren: false },
  { key: 'Divider', label: 'Divider', acceptsChildren: false },
  { key: 'ProviderButtons', label: 'Sign-in Providers', acceptsChildren: false },
]

const TARGETS_BY_KEY: ReadonlyMap<string, FluidRenderTarget> = new Map(
  RENDER_TARGETS.map((target) => [target.key, target]),
)

/** The render target a rendered node maps to, if it is one we know. */
export function renderTargetOfNode(
  node: Pick<ThemeNode, 'type' | 'component'>,
): FluidRenderTarget | undefined {
  return TARGETS_BY_KEY.get(node.component ?? node.type)
}

export interface FluidBlockDefinition {
  id: FluidBlockId
  label: string
  description: string
  icon: LucideIcon
  category: BlockCategory
  node: ThemeNodeDefinition
}

export const FLUID_BLOCKS: readonly FluidBlockDefinition[] = [
  {
    id: FluidBlockId.Box,
    label: 'Box',
    description: 'Container with auto-layout',
    icon: LayoutTemplate,
    category: BlockCategory.Layout,
    node: {
      type: 'Box',
      size: { width: 'fill', height: 'hug' },
      layout: { direction: 'column', gap: 12, align: 'stretch', padding: [0, 0, 0, 0] },
      children: [],
    },
  },
  {
    id: FluidBlockId.Text,
    label: 'Text',
    description: 'Headings, labels, hints',
    icon: Type,
    category: BlockCategory.Text,
    node: {
      type: 'Text',
      size: { width: 'fill', height: 'hug' },
      props: { text: 'Welcome back' },
    },
  },
  {
    id: FluidBlockId.Input,
    label: 'Input Field',
    description: 'Email, password, custom',
    icon: LayoutTemplate,
    category: BlockCategory.FormElements,
    node: {
      type: 'Component',
      component: 'Input',
      size: { width: 'fill', height: 'hug' },
      props: { label: 'Email', name: 'email', input_type: 'text' },
      slots: {
        prefix: {
          type: 'Icon',
          size: { width: 'hug', height: 'hug' },
          props: { name: 'mail', color: '#94a3b8', size: 14, visible: false },
        },
        error: {
          type: 'Text',
          size: { width: 'fill', height: 'hug' },
          props: { text: 'Invalid value', color: '#ef4444', visible: false },
        },
      },
    },
  },
  {
    id: FluidBlockId.Button,
    label: 'Button',
    description: 'Primary, secondary actions',
    icon: MousePointer2,
    category: BlockCategory.Actions,
    node: {
      type: 'Component',
      component: 'Button',
      size: { width: 'fill', height: 'hug' },
      props: { label: 'Continue', variant: 'primary' },
    },
  },
  {
    id: FluidBlockId.ProviderButtons,
    label: 'Sign-in Providers',
    description: 'Buttons for the realm’s enabled providers',
    icon: Fingerprint,
    category: BlockCategory.Actions,
    node: {
      type: 'Component',
      component: 'ProviderButtons',
      size: { width: 'fill', height: 'hug' },
      // The runtime hides the block when the realm has no enabled providers.
      props: { visible_if: 'enabled_providers_count' },
    },
  },
  {
    id: FluidBlockId.Divider,
    label: 'Divider',
    description: 'Section separators',
    icon: Minus,
    category: BlockCategory.Layout,
    node: {
      type: 'Component',
      component: 'Divider',
      size: { width: 'fill', height: 'hug' },
      props: {},
    },
  },
  {
    id: FluidBlockId.Link,
    label: 'Link',
    description: 'Inline navigation or legal links',
    icon: Type,
    category: BlockCategory.Text,
    node: {
      type: 'Component',
      component: 'Link',
      size: { width: 'fill', height: 'hug' },
      props: { label: 'Forgot password?', href: '/forgot-password', target: '_self' },
    },
  },
  {
    id: FluidBlockId.Image,
    label: 'Image',
    description: 'Brand or hero image',
    icon: Image,
    category: BlockCategory.Media,
    node: {
      type: 'Image',
      size: { width: 'fill', height: 'hug' },
      props: { asset_id: '', alt: 'Brand image' },
    },
  },

  {
    id: FluidBlockId.Columns,
    label: 'Columns',
    description: 'Row container for side-by-side blocks',
    icon: Columns2,
    category: BlockCategory.Layout,
    // A Box with its direction preset. The catalog is a palette of starting
    // points, so two entries may share a render target.
    node: {
      type: 'Box',
      size: { width: 'fill', height: 'hug' },
      layout: { direction: 'row', gap: 12, align: 'center', justify: 'between', padding: [0, 0, 0, 0] },
      children: [],
    },
  },
  {
    id: FluidBlockId.Heading,
    label: 'Heading',
    description: 'Page title, preset larger',
    icon: Heading1,
    category: BlockCategory.Text,
    node: {
      type: 'Text',
      size: { width: 'fill', height: 'hug' },
      props: { text: 'Welcome back' },
      style: { typography: { size: '24px', weight: '700' } },
    },
  },
  {
    id: FluidBlockId.Checkbox,
    label: 'Checkbox',
    description: 'Consent, remember me, opt-in',
    icon: SquareCheck,
    category: BlockCategory.FormElements,
    node: {
      type: 'Component',
      component: 'Checkbox',
      size: { width: 'fill', height: 'hug' },
      props: { label: 'Remember me', name: 'remember_me', checked: false },
    },
  },
  {
    id: FluidBlockId.RadioGroup,
    label: 'Radio Group',
    description: 'Single choice from a short list',
    icon: Circle,
    category: BlockCategory.FormElements,
    node: {
      type: 'Component',
      component: 'RadioGroup',
      size: { width: 'fill', height: 'hug' },
      props: { name: 'choice', options: 'yes|Yes\nno|No', value: '' },
    },
  },
  {
    id: FluidBlockId.Select,
    label: 'Select',
    description: 'Dropdown for a longer list',
    icon: ChevronDown,
    category: BlockCategory.FormElements,
    node: {
      type: 'Component',
      component: 'Select',
      size: { width: 'fill', height: 'hug' },
      props: {
        name: 'choice',
        placeholder: 'Choose one',
        options: 'yes|Yes\nno|No',
        value: '',
      },
    },
  },
  {
    id: FluidBlockId.LegalText,
    label: 'Legal Text',
    description: 'Consent copy with inline links',
    icon: FileText,
    category: BlockCategory.Text,
    node: {
      type: 'Component',
      component: 'LegalText',
      size: { width: 'fill', height: 'hug' },
      props: { text: 'I accept the [Terms](/terms) and [Privacy Policy](/privacy).' },
    },
  },
]

export function findBlockDefinition(id: FluidBlockId): FluidBlockDefinition | undefined {
  return FLUID_BLOCKS.find((block) => block.id === id)
}

export function labelForNode(node: Pick<ThemeNode, 'type' | 'component'>): string {
  return renderTargetOfNode(node)?.label ?? node.component ?? node.type
}

/**
 * Whether a node may contain children.
 *
 * Unknown node keys answer `false`: an unrecognised block is never a drop
 * target, which keeps a hand-edited blueprint from opening a nesting hole.
 */
export function canAcceptChildren(node: Pick<ThemeNode, 'type' | 'component'>): boolean {
  return renderTargetOfNode(node)?.acceptsChildren ?? false
}

export function filterBlocks(query: string): readonly FluidBlockDefinition[] {
  const normalized = query.trim().toLowerCase()
  if (!normalized) return FLUID_BLOCKS
  return FLUID_BLOCKS.filter(
    (block) =>
      block.label.toLowerCase().includes(normalized) ||
      block.description.toLowerCase().includes(normalized),
  )
}

export interface BlockCategoryGroup {
  category: BlockCategory
  blocks: FluidBlockDefinition[]
}

/** Groups blocks into `BLOCK_CATEGORY_ORDER`, dropping empty categories. */
export function groupBlocksByCategory(
  blocks: readonly FluidBlockDefinition[],
): BlockCategoryGroup[] {
  return BLOCK_CATEGORY_ORDER.map((category) => ({
    category,
    blocks: blocks.filter((block) => block.category === category),
  })).filter((group) => group.blocks.length > 0)
}

export function buildFluidNode(definition: FluidBlockDefinition): ThemeNode {
  return createNodeFromDefinition(definition.node)
}
