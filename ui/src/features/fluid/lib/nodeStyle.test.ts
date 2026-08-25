import { describe, expect, it } from 'vitest'

import type { ThemeNode } from '@/entities/theme/model/types'
import {
  LEGACY_PART_PROPS,
  LEGACY_STYLE_PROPS,
  normalizeNode,
  normalizeNodes,
  readStyleValue,
  resolveNodeStyle,
  resolvePartStyle,
  withStyleValue,
} from './nodeStyle'

describe('resolveNodeStyle', () => {
  it('reads a style group', () => {
    const node: ThemeNode = { id: 'a', type: 'Box', style: { fill: { color: '#101828' } } }
    expect(resolveNodeStyle(node).fill.color).toBe('#101828')
  })

  it('falls back to the legacy prop so stored blueprints keep rendering', () => {
    const node: ThemeNode = { id: 'a', type: 'Box', props: { background: '#101828' } }
    expect(resolveNodeStyle(node).fill.color).toBe('#101828')
  })

  it('prefers the style group when both are present', () => {
    const node: ThemeNode = {
      id: 'a',
      type: 'Box',
      props: { background: '#legacy' },
      style: { fill: { color: '#group' } },
    }
    expect(resolveNodeStyle(node).fill.color).toBe('#group')
  })

  it('returns every group, so callers never null-check one', () => {
    const resolved = resolveNodeStyle({ id: 'a', type: 'Text' } as ThemeNode)
    expect(Object.keys(resolved).sort()).toEqual([
      'corners',
      'fill',
      'spacing',
      'stroke',
      'typography',
    ])
  })

  it('ignores an empty legacy value rather than emitting a blank', () => {
    const node: ThemeNode = { id: 'a', type: 'Box', props: { background: '' } }
    expect(resolveNodeStyle(node).fill.color).toBeUndefined()
  })

  it('maps every legacy styling prop to a group', () => {
    const props = Object.fromEntries(
      Object.keys(LEGACY_STYLE_PROPS).map((key) => [key, 'set']),
    )
    const resolved = resolveNodeStyle({ id: 'a', type: 'Box', props } as ThemeNode)
    for (const [group, key] of Object.values(LEGACY_STYLE_PROPS)) {
      expect((resolved[group] as Record<string, unknown>)[key]).toBe('set')
    }
  })
})

describe('resolvePartStyle', () => {
  const inputWithLegacy: ThemeNode = {
    id: 'field',
    type: 'Component',
    component: 'Input',
    props: {
      label: 'Email',
      label_color: '#8899aa',
      label_size: '12px',
      field_background: '#000000',
      field_radius: 6,
    },
  }

  it('folds the prefixed props into their part', () => {
    expect(resolvePartStyle(inputWithLegacy, 'label').typography).toMatchObject({
      color: '#8899aa',
      size: '12px',
    })
    expect(resolvePartStyle(inputWithLegacy, 'field').fill.color).toBe('#000000')
    expect(resolvePartStyle(inputWithLegacy, 'field').corners.radius).toBe(6)
  })

  it('keeps a part isolated from the others', () => {
    expect(resolvePartStyle(inputWithLegacy, 'label').fill.color).toBeUndefined()
  })

  it('prefers an explicit part style over the legacy prop', () => {
    const node: ThemeNode = {
      ...inputWithLegacy,
      style: { parts: { field: { fill: { color: '#ffffff' } } } },
    }
    expect(resolvePartStyle(node, 'field').fill.color).toBe('#ffffff')
  })

  it('covers all nine prefixed props', () => {
    expect(Object.keys(LEGACY_PART_PROPS)).toHaveLength(9)
  })
})

describe('normalizeNode', () => {
  it('moves legacy props into groups and drops them', () => {
    const node: ThemeNode = {
      id: 'a',
      type: 'Box',
      props: { background: '#101828', border_width: 2, text: 'kept' },
    }
    const next = normalizeNode(node)
    expect(next.style).toEqual({ fill: { color: '#101828' }, stroke: { width: 2 } })
    expect(next.props).toEqual({ text: 'kept' })
  })

  it('moves prefixed props into parts', () => {
    const node: ThemeNode = {
      id: 'f',
      type: 'Component',
      component: 'Input',
      props: { name: 'email', label_color: '#abc', field_padding: 10 },
    }
    const next = normalizeNode(node)
    expect(next.style?.parts).toEqual({
      label: { typography: { color: '#abc' } },
      field: { spacing: { padding: 10 } },
    })
    expect(next.props).toEqual({ name: 'email' })
  })

  it('recurses through children and slots', () => {
    const node: ThemeNode = {
      id: 'root',
      type: 'Box',
      children: [{ id: 'child', type: 'Text', props: { color: '#fff' } }],
      slots: { prefix: { id: 'p', type: 'Icon', props: { color: '#eee', name: 'mail' } } },
    }
    const next = normalizeNode(node)
    expect(next.children?.[0].style?.typography?.color).toBe('#fff')
    expect(next.slots?.prefix.style?.typography?.color).toBe('#eee')
    expect(next.slots?.prefix.props).toEqual({ name: 'mail' })
  })

  it('returns the same node when there is nothing to convert', () => {
    const node: ThemeNode = { id: 'a', type: 'Text', props: { text: 'Hi' } }
    expect(normalizeNode(node)).toBe(node)
  })

  it('keeps an unknown style group rather than deleting authored data', () => {
    const node = {
      id: 'a',
      type: 'Box',
      props: { background: '#111' },
      style: { glow: { spread: 4 } },
    } as unknown as ThemeNode
    const next = normalizeNode(node) as unknown as Record<string, Record<string, unknown>>
    expect(next.style.glow).toEqual({ spread: 4 })
    expect(next.style.fill).toEqual({ color: '#111' })
  })

  it('does not overwrite a group value that is already set', () => {
    const node: ThemeNode = {
      id: 'a',
      type: 'Box',
      props: { background: '#legacy' },
      style: { fill: { color: '#group' } },
    }
    const next = normalizeNode(node)
    expect(next.style?.fill?.color).toBe('#group')
    expect(next.props?.background).toBeUndefined()
  })

  it('leaves the source node untouched', () => {
    const node: ThemeNode = { id: 'a', type: 'Box', props: { background: '#101828' } }
    const before = JSON.stringify(node)
    normalizeNode(node)
    expect(JSON.stringify(node)).toBe(before)
  })
})

describe('normalizeNodes', () => {
  it('keeps array identity when nothing changed', () => {
    const nodes: ThemeNode[] = [{ id: 'a', type: 'Text', props: { text: 'Hi' } }]
    expect(normalizeNodes(nodes)).toBe(nodes)
  })

  it('converts a whole page in one pass', () => {
    const nodes: ThemeNode[] = [
      { id: 'a', type: 'Box', props: { radius: 8 } },
      { id: 'b', type: 'Text', props: { font_size: '14px' } },
    ]
    const next = normalizeNodes(nodes)
    expect(next[0].style?.corners?.radius).toBe(8)
    expect(next[1].style?.typography?.size).toBe('14px')
  })
})

describe('withStyleValue', () => {
  it('writes into the group', () => {
    const node: ThemeNode = { id: 'a', type: 'Box' }
    expect(withStyleValue(node, 'fill', 'color', '#123456').style?.fill?.color).toBe('#123456')
  })

  it('clears the value and the group when blank', () => {
    const node: ThemeNode = { id: 'a', type: 'Box', style: { fill: { color: '#123456' } } }
    expect(withStyleValue(node, 'fill', 'color', '').style).toBeUndefined()
  })

  it('drops the matching legacy prop so a node never carries both', () => {
    const node: ThemeNode = { id: 'a', type: 'Box', props: { background: '#old', text: 'keep' } }
    const next = withStyleValue(node, 'fill', 'color', '#new')
    expect(next.style?.fill?.color).toBe('#new')
    expect(next.props).toEqual({ text: 'keep' })
  })

  it('round-trips through readStyleValue', () => {
    const node = withStyleValue({ id: 'a', type: 'Box' }, 'corners', 'radius', 12)
    expect(readStyleValue(node, 'corners', 'radius')).toBe(12)
  })
})
