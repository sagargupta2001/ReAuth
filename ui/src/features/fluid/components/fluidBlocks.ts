import { Image, LayoutTemplate, Minus, MousePointer2, Type } from 'lucide-react'

import type { ThemeNode } from '@/entities/theme/model/types'
import {
  createNodeFromDefinition,
  type ThemeNodeDefinition,
} from '@/features/fluid/lib/nodeUtils'

export interface FluidBlockDefinition {
  id: string
  label: string
  description: string
  icon: typeof LayoutTemplate
  category: string
  node: ThemeNodeDefinition
}

export const FLUID_BLOCKS: FluidBlockDefinition[] = [
  {
    id: 'box',
    label: 'Box',
    description: 'Container with auto-layout',
    icon: LayoutTemplate,
    category: 'Layout',
    node: {
      type: 'Box',
      size: { width: 'fill', height: 'hug' },
      layout: { direction: 'column', gap: 12, align: 'stretch', padding: [0, 0, 0, 0] },
      children: [],
    },
  },
  {
    id: 'text',
    label: 'Text',
    description: 'Headings, labels, hints',
    icon: Type,
    category: 'Text',
    node: {
      type: 'Text',
      size: { width: 'fill', height: 'hug' },
      props: { text: 'Welcome back' },
    },
  },
  {
    id: 'input',
    label: 'Input Field',
    description: 'Email, password, custom',
    icon: LayoutTemplate,
    category: 'Form Elements',
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
    id: 'button',
    label: 'Button',
    description: 'Primary, secondary actions',
    icon: MousePointer2,
    category: 'Actions',
    node: {
      type: 'Component',
      component: 'Button',
      size: { width: 'fill', height: 'hug' },
      props: { label: 'Continue', variant: 'primary' },
    },
  },
  {
    id: 'divider',
    label: 'Divider',
    description: 'Section separators',
    icon: Minus,
    category: 'Layout',
    node: {
      type: 'Component',
      component: 'Divider',
      size: { width: 'fill', height: 'hug' },
      props: {},
    },
  },
  {
    id: 'link',
    label: 'Link',
    description: 'Inline navigation or legal links',
    icon: Type,
    category: 'Text',
    node: {
      type: 'Component',
      component: 'Link',
      size: { width: 'fill', height: 'hug' },
      props: { label: 'Forgot password?', href: '/forgot-password', target: '_self' },
    },
  },
  {
    id: 'image',
    label: 'Image',
    description: 'Brand or hero image',
    icon: Image,
    category: 'Media',
    node: {
      type: 'Image',
      size: { width: 'fill', height: 'hug' },
      props: { asset_id: '', alt: 'Brand image' },
    },
  },
]

export function buildFluidNode(definition: FluidBlockDefinition): ThemeNode {
  return createNodeFromDefinition(definition.node)
}
