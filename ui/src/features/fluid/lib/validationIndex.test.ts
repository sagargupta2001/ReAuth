import { describe, expect, it } from 'vitest'

import { buildValidationIndex } from './validationIndex'

describe('buildValidationIndex', () => {
  it('splits node errors from page errors', () => {
    const { byNodeId, pageErrors } = buildValidationIndex([
      { path: 'blueprint', message: 'Blueprint must define a nodes array.' },
      { path: 'nodes[0]', message: 'Node type is required.', nodeId: 'a' },
      { path: 'nodes[0]', message: 'Unsupported node type', nodeId: 'a' },
      { path: 'nodes[1]', message: 'Node type is required.', nodeId: 'b' },
    ])

    expect(pageErrors).toHaveLength(1)
    expect(byNodeId.get('a')).toHaveLength(2)
    expect(byNodeId.get('b')).toHaveLength(1)
    expect(byNodeId.has('c')).toBe(false)
  })

  it('returns empty collections for no errors', () => {
    const { byNodeId, pageErrors } = buildValidationIndex([])
    expect(byNodeId.size).toBe(0)
    expect(pageErrors).toEqual([])
  })
})
