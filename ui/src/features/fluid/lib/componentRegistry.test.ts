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

describe('expandComponentNode: Input', () => {
  it('derives the field border and label colour from the theme text colour', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#ededed' })

    expect(fieldOf(expanded)?.props?.border_color).toBe('rgba(237, 237, 237, 0.22)')
    expect(labelOf(expanded)?.props?.color).toBe('rgba(237, 237, 237, 0.7)')
  })

  it('adapts to a light theme instead of hard-coding light values', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#111827' })

    expect(fieldOf(expanded)?.props?.border_color).toBe('rgba(17, 24, 39, 0.22)')
  })

  it('leaves the field transparent so it sits on the theme surface', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#ededed' })
    expect(fieldOf(expanded)?.props?.background).toBe('transparent')
  })

  it('takes the corner radius from the theme', () => {
    const expanded = expandComponentNode(inputNode(), { text: '#ededed', radius: 16 })
    expect(fieldOf(expanded)?.props?.radius).toBe(16)
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

    expect(fieldOf(expanded)?.props?.border_color).toBe('#ff0000')
    expect(fieldOf(expanded)?.props?.background).toBe('#00ff00')
    expect(fieldOf(expanded)?.props?.radius).toBe(2)
    expect(labelOf(expanded)?.props?.color).toBe('#0000ff')
  })

  it('falls back to a neutral border when no theme is supplied', () => {
    const expanded = expandComponentNode(inputNode())
    expect(fieldOf(expanded)?.props?.border_color).toBe('rgba(127, 127, 127, 0.22)')
  })

  it('returns null for components it does not know', () => {
    expect(
      expandComponentNode({ id: 'x', type: 'Component', component: 'Nope' }),
    ).toBeNull()
    expect(expandComponentNode({ id: 'x', type: 'Text' })).toBeNull()
  })
})
