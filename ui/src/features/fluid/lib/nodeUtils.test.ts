import { describe, expect, it } from 'vitest'

import type { ThemeNode } from '@/entities/theme/model/types'
import {
  depthOfNode,
  findNodePath,
  insertNodeAt,
  isDescendantOf,
  locationOfNode,
  moveNode,
  resolveDrop,
  subtreeHeight,
  type DropOptions,
} from './nodeUtils'
import { canAcceptChildren } from '@/features/fluid/model/blockCatalog'
import { MAX_NESTING_DEPTH } from '@/features/fluid/model/sectionTree'

const OPTIONS: DropOptions = {
  acceptsChildren: canAcceptChildren,
  maxDepth: MAX_NESTING_DEPTH,
}

function text(id: string): ThemeNode {
  return { id, type: 'Text', props: { text: id } }
}

function box(id: string, children: ThemeNode[] = []): ThemeNode {
  return { id, type: 'Box', children }
}

/**
 *  heading
 *  outer
 *    inner
 *      deep
 *    sibling
 *  field  (Component:Input, with a `prefix` slot)
 */
function sampleTree(): ThemeNode[] {
  return [
    text('heading'),
    box('outer', [box('inner', [text('deep')]), text('sibling')]),
    {
      id: 'field',
      type: 'Component',
      component: 'Input',
      props: { label: 'Email' },
      slots: { prefix: { id: 'field-prefix', type: 'Icon', props: { name: 'mail' } } },
    },
  ]
}

const idsOf = (nodes: ThemeNode[]): string[] => nodes.map((node) => node.id)

describe('findNodePath', () => {
  it('returns the root-to-node chain', () => {
    expect(idsOf(findNodePath(sampleTree(), 'deep') ?? [])).toEqual(['outer', 'inner', 'deep'])
  })

  it('returns null for an unknown id', () => {
    expect(findNodePath(sampleTree(), 'nope')).toBeNull()
  })

  it('does not descend into slots', () => {
    // Slot children are a component's contract, not free composition. Returning
    // null here is what makes every structural operation refuse them without
    // each call site re-checking.
    expect(findNodePath(sampleTree(), 'field-prefix')).toBeNull()
  })
})

describe('depthOfNode', () => {
  it('counts root nodes as depth 0', () => {
    expect(depthOfNode(sampleTree(), 'heading')).toBe(0)
    expect(depthOfNode(sampleTree(), 'inner')).toBe(1)
    expect(depthOfNode(sampleTree(), 'deep')).toBe(2)
  })
})

describe('subtreeHeight', () => {
  it('is zero for a leaf and counts levels below a container', () => {
    expect(subtreeHeight(text('a'))).toBe(0)
    expect(subtreeHeight(box('a', [box('b', [text('c')])]))).toBe(2)
  })
})

describe('isDescendantOf', () => {
  it('is true for any node inside the ancestor subtree', () => {
    expect(isDescendantOf(sampleTree(), 'deep', 'outer')).toBe(true)
    expect(isDescendantOf(sampleTree(), 'deep', 'inner')).toBe(true)
  })

  it('is false for the node itself and for unrelated nodes', () => {
    expect(isDescendantOf(sampleTree(), 'outer', 'outer')).toBe(false)
    expect(isDescendantOf(sampleTree(), 'heading', 'outer')).toBe(false)
  })
})

describe('locationOfNode', () => {
  it('addresses a root node against the page list', () => {
    expect(locationOfNode(sampleTree(), 'outer')).toEqual({ parentId: null, index: 1 })
  })

  it('addresses a child against its parent', () => {
    expect(locationOfNode(sampleTree(), 'sibling')).toEqual({ parentId: 'outer', index: 1 })
  })

  it('returns null for a slot node', () => {
    expect(locationOfNode(sampleTree(), 'field-prefix')).toBeNull()
  })
})

describe('insertNodeAt', () => {
  it('inserts into the page root', () => {
    const next = insertNodeAt(sampleTree(), text('new'), { parentId: null, index: 1 })
    expect(idsOf(next)).toEqual(['heading', 'new', 'outer', 'field'])
  })

  it('inserts into a container', () => {
    const next = insertNodeAt(sampleTree(), text('new'), { parentId: 'inner', index: 0 })
    expect(idsOf(findNodePath(next, 'new') ?? [])).toEqual(['outer', 'inner', 'new'])
  })

  it('clamps an out-of-range index instead of dropping the node', () => {
    const next = insertNodeAt(sampleTree(), text('new'), { parentId: null, index: 99 })
    expect(idsOf(next).at(-1)).toBe('new')
  })

  it('leaves the tree untouched for an unknown parent', () => {
    const nodes = sampleTree()
    expect(insertNodeAt(nodes, text('new'), { parentId: 'nope', index: 0 })).toBe(nodes)
  })
})

describe('moveNode', () => {
  it('nests a root node into a container', () => {
    const next = moveNode(sampleTree(), 'heading', { parentId: 'inner', index: 1 })
    expect(idsOf(next)).toEqual(['outer', 'field'])
    expect(idsOf(findNodePath(next, 'heading') ?? [])).toEqual(['outer', 'inner', 'heading'])
  })

  it('reorders within a container', () => {
    const next = moveNode(sampleTree(), 'sibling', { parentId: 'outer', index: 0 })
    expect(idsOf(findNodePath(next, 'outer')?.at(-1)?.children ?? [])).toEqual([
      'sibling',
      'inner',
    ])
  })

  it('shifts the target index when moving forward within the same parent', () => {
    // The location is expressed against the tree *before* the node is detached.
    const nodes = [text('a'), text('b'), text('c')]
    expect(idsOf(moveNode(nodes, 'a', { parentId: null, index: 3 }))).toEqual(['b', 'c', 'a'])
  })

  it('moves a child out to the page root and leaves the container in place', () => {
    const nodes = [box('outer', [text('only')]), text('tail')]
    const next = moveNode(nodes, 'only', { parentId: null, index: 2 })
    expect(idsOf(next)).toEqual(['outer', 'tail', 'only'])
    expect(next[0].children).toEqual([])
  })

  it('refuses to move a node inside its own subtree', () => {
    const nodes = sampleTree()
    expect(moveNode(nodes, 'outer', { parentId: 'inner', index: 0 })).toBe(nodes)
  })

  it('ignores an id that is no longer on the page', () => {
    const nodes = sampleTree()
    expect(moveNode(nodes, 'removed', { parentId: null, index: 0 })).toBe(nodes)
  })

  it('ignores a move that lands where the node already is', () => {
    const nodes = sampleTree()
    expect(moveNode(nodes, 'heading', { parentId: null, index: 0 })).toBe(nodes)
  })

  it('leaves the source tree untouched', () => {
    // Undo restores a previously committed tree by reference, so a structural
    // edit that mutated in place would corrupt every entry in the history.
    const nodes = sampleTree()
    const before = JSON.stringify(nodes)

    moveNode(nodes, 'heading', { parentId: 'inner', index: 0 })
    insertNodeAt(nodes, text('new'), { parentId: 'outer', index: 0 })

    expect(JSON.stringify(nodes)).toBe(before)
  })
})

describe('resolveDrop', () => {
  it('resolves a drop on a row top edge to the sibling above it', () => {
    const resolution = resolveDrop(
      sampleTree(),
      { dragId: 'field', targetId: 'heading', intent: 'before' },
      OPTIONS,
    )
    expect(resolution).toEqual({
      ok: true,
      intent: 'before',
      location: { parentId: null, index: 0 },
    })
  })

  it('resolves a drop on a container middle to its last child slot', () => {
    const resolution = resolveDrop(
      sampleTree(),
      { dragId: 'heading', targetId: 'outer', intent: 'inside' },
      OPTIONS,
    )
    expect(resolution).toEqual({
      ok: true,
      intent: 'inside',
      location: { parentId: 'outer', index: 2 },
    })
  })

  it('treats a nesting drop on a non-container as a sibling drop', () => {
    const resolution = resolveDrop(
      sampleTree(),
      { dragId: 'field', targetId: 'heading', intent: 'inside' },
      OPTIONS,
    )
    expect(resolution).toEqual({
      ok: true,
      intent: 'after',
      location: { parentId: null, index: 1 },
    })
  })

  it('rejects a drop into the dragged node itself', () => {
    const resolution = resolveDrop(
      sampleTree(),
      { dragId: 'outer', targetId: 'outer', intent: 'inside' },
      OPTIONS,
    )
    expect(resolution).toMatchObject({ ok: false, reason: 'cycle' })
  })

  it('rejects a drop into a descendant of the dragged node', () => {
    const resolution = resolveDrop(
      sampleTree(),
      { dragId: 'outer', targetId: 'inner', intent: 'inside' },
      OPTIONS,
    )
    expect(resolution).toMatchObject({ ok: false, reason: 'cycle' })
  })

  it('rejects a sibling drop whose parent is inside the dragged subtree', () => {
    const resolution = resolveDrop(
      sampleTree(),
      { dragId: 'outer', targetId: 'deep', intent: 'after' },
      OPTIONS,
    )
    expect(resolution).toMatchObject({ ok: false, reason: 'cycle' })
  })

  it('rejects a drop that would exceed the nesting limit', () => {
    let chain = box(`box-${MAX_NESTING_DEPTH - 1}`)
    for (let level = MAX_NESTING_DEPTH - 2; level >= 0; level -= 1) {
      chain = box(`box-${level}`, [chain])
    }
    const nodes = [chain, text('loose')]
    const deepest = `box-${MAX_NESTING_DEPTH - 1}`

    expect(depthOfNode(nodes, deepest)).toBe(MAX_NESTING_DEPTH - 1)
    const resolution = resolveDrop(
      nodes,
      { dragId: 'loose', targetId: deepest, intent: 'inside' },
      OPTIONS,
    )
    expect(resolution).toMatchObject({ ok: false, reason: 'depth-limit' })
    expect(resolution.ok ? '' : resolution.message).toContain(String(MAX_NESTING_DEPTH))
  })

  it('measures the dragged subtree, not just the dragged node', () => {
    const nodes = [box('a', [box('b')]), box('tall', [box('t1', [box('t2')])])]
    // `tall` is 2 levels deep on its own, so nesting it under `b` (depth 1)
    // would put `t2` at depth 4.
    expect(
      resolveDrop(nodes, { dragId: 'tall', targetId: 'b', intent: 'inside' }, {
        ...OPTIONS,
        maxDepth: 4,
      }),
    ).toMatchObject({ ok: false, reason: 'depth-limit' })
    expect(
      resolveDrop(nodes, { dragId: 'tall', targetId: 'b', intent: 'inside' }, {
        ...OPTIONS,
        maxDepth: 5,
      }),
    ).toMatchObject({ ok: true })
  })

  it('rejects a drop that lands where the node already is', () => {
    const nodes = [text('a'), text('b')]
    expect(
      resolveDrop(nodes, { dragId: 'a', targetId: 'b', intent: 'before' }, OPTIONS),
    ).toMatchObject({ ok: false, reason: 'no-op' })
    expect(
      resolveDrop(nodes, { dragId: 'b', targetId: 'a', intent: 'after' }, OPTIONS),
    ).toMatchObject({ ok: false, reason: 'no-op' })
  })

  it('rejects a drag whose node was removed before the drop', () => {
    expect(
      resolveDrop(sampleTree(), { dragId: 'gone', targetId: 'heading', intent: 'after' }, OPTIONS),
    ).toMatchObject({ ok: false, reason: 'unknown-node' })
  })

  it('rejects a drop onto a slot row', () => {
    expect(
      resolveDrop(
        sampleTree(),
        { dragId: 'heading', targetId: 'field-prefix', intent: 'after' },
        OPTIONS,
      ),
    ).toMatchObject({ ok: false, reason: 'unknown-node' })
  })

  it('rejects dragging a slot node', () => {
    expect(
      resolveDrop(
        sampleTree(),
        { dragId: 'field-prefix', targetId: 'heading', intent: 'after' },
        OPTIONS,
      ),
    ).toMatchObject({ ok: false, reason: 'unknown-node' })
  })

  it('appends to the page root when there is no target row', () => {
    expect(
      resolveDrop(sampleTree(), { dragId: 'heading', targetId: null, intent: 'inside' }, OPTIONS),
    ).toMatchObject({ ok: true, location: { parentId: null, index: 3 } })
  })
})
