import { describe, expect, it } from 'vitest'

import {
  BLOCK_CATEGORY_ORDER,
  FLUID_BLOCKS,
  FluidBlockId,
  buildFluidNode,
  canAcceptChildren,
  filterBlocks,
  findBlockDefinition,
  groupBlocksByCategory,
  labelForNode,
} from './blockCatalog'

describe('FLUID_BLOCKS', () => {
  it('has a unique id per block', () => {
    const ids = FLUID_BLOCKS.map((block) => block.id)
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('covers every declared block id', () => {
    const ids = new Set(FLUID_BLOCKS.map((block) => block.id))
    Object.values(FluidBlockId).forEach((id) => expect(ids.has(id)).toBe(true))
  })

  it('only uses categories that have a display position', () => {
    FLUID_BLOCKS.forEach((block) => {
      expect(BLOCK_CATEGORY_ORDER).toContain(block.category)
    })
  })
})

describe('canAcceptChildren', () => {
  it('accepts children for Box and nothing else in this slice', () => {
    const containers = FLUID_BLOCKS.filter((block) => block.acceptsChildren).map(
      (block) => block.id,
    )
    expect(containers).toEqual([FluidBlockId.Box])
  })

  it('answers from the rendered node, by component name or type', () => {
    expect(canAcceptChildren({ type: 'Box' })).toBe(true)
    expect(canAcceptChildren({ type: 'Text' })).toBe(false)
    expect(canAcceptChildren({ type: 'Component', component: 'Input' })).toBe(false)
    expect(canAcceptChildren({ type: 'Component', component: 'Link' })).toBe(false)
  })

  it('refuses an unrecognised node rather than opening a nesting hole', () => {
    expect(canAcceptChildren({ type: 'Component', component: 'Unknown' })).toBe(false)
  })
})

describe('filterBlocks', () => {
  it('returns the full catalog for a blank query', () => {
    expect(filterBlocks('   ')).toHaveLength(FLUID_BLOCKS.length)
  })

  it('matches labels case-insensitively', () => {
    expect(filterBlocks('DIVIDER').map((block) => block.id)).toEqual([FluidBlockId.Divider])
  })

  it('matches descriptions', () => {
    expect(filterBlocks('auto-layout').map((block) => block.id)).toEqual([FluidBlockId.Box])
  })

  it('returns every block that matches the query', () => {
    expect(filterBlocks('button').map((block) => block.id)).toEqual([
      FluidBlockId.Button,
      FluidBlockId.ProviderButtons,
    ])
  })

  it('returns nothing for an unknown query', () => {
    expect(filterBlocks('no-such-block')).toEqual([])
  })
})

describe('groupBlocksByCategory', () => {
  it('groups in the declared category order and drops empty groups', () => {
    const groups = groupBlocksByCategory(filterBlocks('divider'))
    expect(groups).toHaveLength(1)
    expect(groups[0].blocks.map((block) => block.id)).toEqual([FluidBlockId.Divider])
  })

  it('orders full-catalog groups by BLOCK_CATEGORY_ORDER', () => {
    const categories = groupBlocksByCategory(FLUID_BLOCKS).map((group) => group.category)
    expect(categories).toEqual(
      BLOCK_CATEGORY_ORDER.filter((category) =>
        FLUID_BLOCKS.some((block) => block.category === category),
      ),
    )
  })
})

describe('labelForNode', () => {
  it('prefers the component name over the node type', () => {
    expect(labelForNode({ type: 'Component', component: 'Button' })).toBe('Button')
    expect(labelForNode({ type: 'Text' })).toBe('Text')
  })

  it('falls back to the raw key for unknown nodes', () => {
    expect(labelForNode({ type: 'Component', component: 'Unknown' })).toBe('Unknown')
  })
})

describe('buildFluidNode', () => {
  it('creates a node with an id from the definition', () => {
    const definition = findBlockDefinition(FluidBlockId.Input)
    expect(definition).toBeDefined()

    const node = buildFluidNode(definition!)
    expect(node.id).toBeTruthy()
    expect(node.component).toBe('Input')
    expect(node.slots?.prefix?.id).toBeTruthy()
    expect(node.slots?.error?.id).toBeTruthy()
  })

  it('creates a distinct node on every call', () => {
    const definition = findBlockDefinition(FluidBlockId.Text)!
    expect(buildFluidNode(definition).id).not.toBe(buildFluidNode(definition).id)
  })
})
