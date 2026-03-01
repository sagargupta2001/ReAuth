import {
  Image,
  LayoutTemplate,
  Minus,
  MousePointer2,
  Type,
} from 'lucide-react'

export const FLUID_BLOCKS = [
  {
    id: 'text',
    label: 'Text',
    description: 'Headings, labels, hints',
    icon: Type,
    category: 'Text',
    props: { text: 'Welcome back' },
  },
  {
    id: 'input',
    label: 'Input Field',
    description: 'Email, password, custom',
    icon: LayoutTemplate,
    category: 'Form Elements',
    props: { label: 'Email', name: 'email', input_type: 'text' },
  },
  {
    id: 'button',
    label: 'Button',
    description: 'Primary, secondary actions',
    icon: MousePointer2,
    category: 'Actions',
    props: { label: 'Continue', variant: 'primary' },
  },
  {
    id: 'divider',
    label: 'Divider',
    description: 'Section separators',
    icon: Minus,
    category: 'Layout',
    props: {},
  },
  {
    id: 'link',
    label: 'Link',
    description: 'Inline navigation or legal links',
    icon: Type,
    category: 'Text',
    props: { label: 'Forgot password?', href: '/forgot-password', target: '_self' },
  },
  {
    id: 'image',
    label: 'Image',
    description: 'Brand or hero image',
    icon: Image,
    category: 'Media',
    props: { asset_id: '', alt: 'Brand image' },
  },
] as const

export type FluidBlockDefinition = (typeof FLUID_BLOCKS)[number]
