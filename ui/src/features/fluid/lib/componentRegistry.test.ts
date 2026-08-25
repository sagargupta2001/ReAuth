import { describe, expect, it } from 'vitest'

import { expandComponentNode } from './componentRegistry'
import type { ThemeNode } from '@/entities/theme/model/types'

function inputNode(props: Record<string, unknown> = {}): ThemeNode {
  return {
    id: 'input-1',
    type: 'Component',
    component: 'Input',
    props: { label: 'Email', name: 'email', input_type: 'text', ...props },
  }
}

/** Finds the bordered field container inside an expanded Input. */
function fieldOf(expanded: ThemeNode | null) {
  return expanded?.children?.find((child) => child.id.endsWith('-field'))
}

function labelOf(expanded: ThemeNode | null) {
  return expanded?.children?.find((child) => child.id.endsWith('-label'))
}

/**
 * The expansion emits grouped `style`, not the flat styling props it used to.
 * Reading through these keeps each assertion about the value rather than the
 * shape.
 */
const fieldBorder = (expanded: ThemeNode | null) => fieldOf(expanded)?.style?.stroke?.color
const fieldBackground = (expanded: ThemeNode | null) => fieldOf(expanded)?.style?.fill?.color
const fieldRadius = (expanded: ThemeNode | null) => fieldOf(expanded)?.style?.corners?.radius
const labelColour = (expanded: ThemeNode | null) => labelOf(expanded)?.style?.typography?.color

describe('expandComponentNode: Input', () => {
  it('derives the field border and label colour from the theme text colour', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#ededed' })

    expect(fieldBorder(expanded)).toBe('rgba(237, 237, 237, 0.22)')
    expect(labelColour(expanded)).toBe('rgba(237, 237, 237, 0.7)')
  })

  it('adapts to a light theme instead of hard-coding light values', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#111827' })

    expect(fieldBorder(expanded)).toBe('rgba(17, 24, 39, 0.22)')
  })

  it('leaves the field transparent so it sits on the theme surface', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#ededed' })
    expect(fieldBackground(expanded)).toBe('transparent')
  })

  it('takes the corner radius from the theme', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#ededed', radius: 16 })
    expect(fieldRadius(expanded)).toBe(16)
  })

  it('still lets a blueprint override any derived value', () => {
    const expanded = expandComponentNode(
      inputNode({
        field_border_color: '#ff0000',
        field_background: '#00ff00',
        field_radius: 2,
        label_color: '#0000ff',
      }),
      { text: '#ededed', radius: 16 },
    )

    expect(fieldBorder(expanded)).toBe('#ff0000')
    expect(fieldBackground(expanded)).toBe('#00ff00')
    expect(fieldRadius(expanded)).toBe(2)
    expect(labelColour(expanded)).toBe('#0000ff')
  })

  it('falls back to a neutral border when no theme is supplied', () => {
    const expanded = expandComponentNode(inputNode())
    expect(fieldBorder(expanded)).toBe('rgba(127, 127, 127, 0.22)')
  })

  it('accepts part styling from style groups, not just the legacy props', () => {
    const expanded = expandComponentNode(
      {
        ...inputNode(),
        style: {
          parts: {
            field: { stroke: { color: '#ff0000' }, corners: { radius: 2 } },
            label: { typography: { color: '#0000ff' } },
          },
        },
      },
      { text: '#ededed', radius: 16 },
    )

    expect(fieldBorder(expanded)).toBe('#ff0000')
    expect(fieldRadius(expanded)).toBe(2)
    expect(labelColour(expanded)).toBe('#0000ff')
  })

  it('prefers a part style over the legacy prop it replaced', () => {
    const expanded = expandComponentNode(
      {
        ...inputNode({ field_border_color: '#legacy' }),
        style: { parts: { field: { stroke: { color: '#modern' } } } },
      },
      { text: '#ededed' },
    )

    expect(fieldBorder(expanded)).toBe('#modern')
  })

  it('returns null for components it does not know', () => {
    expect(
      expandComponentNode({ id: 'x', type: 'Component', component: 'Nope' }),
    ).toBeNull()
    expect(expandComponentNode({ id: 'x', type: 'Text' })).toBeNull()
  })
})
