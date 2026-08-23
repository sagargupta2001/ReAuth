import {
  Fingerprint,
  Image,
  LayoutTemplate,
  Minus,
  MousePointer2,
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

export interface FluidBlockDefinition {
  id: FluidBlockId
  label: string
  description: string
  icon: LucideIcon
  category: BlockCategory
  /**
   * Whether this block can contain other blocks. Declared here so a future
   * container type (Grid, Stack, Columns) is a data change, the same way the
   * inspector schema and the theme settings schema already work.
   */
  acceptsChildren: boolean
  node: ThemeNodeDefinition
}

export const FLUID_BLOCKS: readonly FluidBlockDefinition[] = [
  {
    id: FluidBlockId.Box,
    label: 'Box',
    description: 'Container with auto-layout',
    icon: LayoutTemplate,
    category: BlockCategory.Layout,
    acceptsChildren: true,
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
    acceptsChildren: false,
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
    acceptsChildren: false,
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
    acceptsChildren: false,
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
    acceptsChildren: false,
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
    acceptsChildren: false,
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
    acceptsChildren: false,
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
    acceptsChildren: false,
    node: {
      type: 'Image',
      size: { width: 'fill', height: 'hug' },
      props: { asset_id: '', alt: 'Brand image' },
    },
  },
]

/** Human label for a rendered node, keyed by component name or node type. */
export const BLOCK_LABEL_BY_NODE_KEY: ReadonlyMap<string, string> = new Map(
  FLUID_BLOCKS.map((block) => [block.node.component ?? block.node.type, block.label]),
)

/** Whether a rendered node's block declares that it accepts children. */
export const BLOCK_ACCEPTS_CHILDREN_BY_NODE_KEY: ReadonlyMap<string, boolean> = new Map(
  FLUID_BLOCKS.map((block) => [block.node.component ?? block.node.type, block.acceptsChildren]),
)

export function findBlockDefinition(id: FluidBlockId): FluidBlockDefinition | undefined {
  return FLUID_BLOCKS.find((block) => block.id === id)
}

export function labelForNode(node: Pick<ThemeNode, 'type' | 'component'>): string {
  const key = node.component ?? node.type
  return BLOCK_LABEL_BY_NODE_KEY.get(key) ?? key
}

/**
 * Whether a node may contain children.
 *
 * Unknown node keys answer `false`: an unrecognised block is never a drop
 * target, which keeps a hand-edited blueprint from opening a nesting hole.
 */
export function canAcceptChildren(node: Pick<ThemeNode, 'type' | 'component'>): boolean {
  return BLOCK_ACCEPTS_CHILDREN_BY_NODE_KEY.get(node.component ?? node.type) ?? false
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
