import { describe, expect, it } from 'vitest'

import { INSPECTOR_SECTIONS } from './inspectorSchema'
import {
  FieldTarget,
  InspectorFieldKind,
  matchesNode,
  type InspectorField,
} from './inspectorFields'
import type { ThemeNode } from '@/entities/theme/model/types'

function node(overrides: Partial<ThemeNode> = {}): ThemeNode {
  return { id: 'n1', type: 'Box', ...overrides }
}

function sectionsFor(target: ThemeNode) {
  return INSPECTOR_SECTIONS.filter((section) => matchesNode(section.appliesTo, target))
}

function fieldsFor(target: ThemeNode): InspectorField[] {
  return sectionsFor(target).flatMap((section) => [...section.fields])
}

function labelsFor(target: ThemeNode): string[] {
  return fieldsFor(target)
    .filter((field): field is Extract<InspectorField, { label: string }> => 'label' in field)
    .map((field) => field.label)
}

describe('matchesNode', () => {
  it('matches by node type', () => {
    expect(matchesNode({ types: ['Box'] }, node())).toBe(true)
    expect(matchesNode({ types: ['Text'] }, node())).toBe(false)
  })

  it('matches a component by name', () => {
    const input = node({ type: 'Component', component: 'Input' })
    expect(matchesNode({ components: ['Input'] }, input)).toBe(true)
    expect(matchesNode({ components: ['Button'] }, input)).toBe(false)
  })

  it('does not match a component name against a non-component node', () => {
    expect(matchesNode({ components: ['Input'] }, node({ type: 'Input' }))).toBe(false)
  })

  it('applies to every node when no matcher is given', () => {
    expect(matchesNode(undefined, node())).toBe(true)
  })
})

describe('INSPECTOR_SECTIONS integrity', () => {
  it('has unique section ids', () => {
    const ids = INSPECTOR_SECTIONS.map((section) => section.id)
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('has unique field ids across the whole schema', () => {
    const ids = INSPECTOR_SECTIONS.flatMap((section) => section.fields.map((f) => f.id))
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('gives every targeted field a target and key', () => {
    const targeted = INSPECTOR_SECTIONS.flatMap((section) =>
      section.fields.filter((field) => 'target' in field),
    )
    expect(targeted.length).toBeGreaterThan(0)
    targeted.forEach((field) => {
      expect(Object.values(FieldTarget)).toContain((field as { target: string }).target)
      expect((field as { key: string }).key).toBeTruthy()
    })
  })
})

describe('no two visible fields share a label', () => {
  // "Alignment" appeared twice for one node — once for layout.align and once for
  // props.align — and "Padding" twice for layout.padding and props.padding.
  const cases: Array<[string, ThemeNode]> = [
    ['Box', node({ type: 'Box' })],
    ['Text', node({ type: 'Text' })],
    ['Image', node({ type: 'Image' })],
    ['Icon', node({ type: 'Icon' })],
    ['Input', node({ type: 'Component', component: 'Input' })],
    ['Button', node({ type: 'Component', component: 'Button' })],
    ['Link', node({ type: 'Component', component: 'Link' })],
    ['Divider', node({ type: 'Component', component: 'Divider' })],
  ]

  cases.forEach(([name, target]) => {
    it(`for a ${name}`, () => {
      const labels = labelsFor(target)
      const duplicates = labels.filter((label, index) => labels.indexOf(label) !== index)
      expect(duplicates).toEqual([])
    })
  })
})

describe('section applicability', () => {
  it('does not offer typography for a Box', () => {
    const ids = sectionsFor(node({ type: 'Box' })).map((section) => section.id)
    expect(ids).not.toContain('typography')
  })

  it('offers typography for text-bearing nodes', () => {
    expect(sectionsFor(node({ type: 'Text' })).map((s) => s.id)).toContain('typography')
    expect(
      sectionsFor(node({ type: 'Component', component: 'Link' })).map((s) => s.id),
    ).toContain('typography')
  })

  it('offers auto-layout only for a Box', () => {
    expect(sectionsFor(node({ type: 'Box' })).map((s) => s.id)).toContain('box-layout')
    expect(sectionsFor(node({ type: 'Text' })).map((s) => s.id)).not.toContain('box-layout')
  })

  it('always offers element, size, placement, and spacing', () => {
    const always = ['element', 'size', 'placement', 'spacing']
    ;[node({ type: 'Box' }), node({ type: 'Text' }), node({ type: 'Image' })].forEach((target) => {
      const ids = sectionsFor(target).map((section) => section.id)
      always.forEach((id) => expect(ids).toContain(id))
    })
  })

  it('exposes a placeholder control for inputs, which the old panel lacked', () => {
    const keys = fieldsFor(node({ type: 'Component', component: 'Input' }))
      .filter((field) => 'key' in field)
      .map((field) => (field as { key: string }).key)
    expect(keys).toContain('placeholder')
  })
})

describe('box layout fields target node.layout', () => {
  it('writes direction, gap, align, justify, and padding to layout', () => {
    const section = INSPECTOR_SECTIONS.find((entry) => entry.id === 'box-layout')
    expect(section).toBeDefined()

    const targeted = section!.fields.filter((field) => 'target' in field)
    targeted.forEach((field) => {
      expect((field as { target: string }).target).toBe(FieldTarget.Layout)
    })

    // The padding tuple has its own control rather than a generic target.
    const padding = section!.fields.find(
      (field) => field.kind === InspectorFieldKind.PaddingBox,
    )
    expect(padding).toBeDefined()
  })
})
