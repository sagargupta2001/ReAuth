import { describe, expect, it } from 'vitest'

import { DROP_EDGE_RATIO, MAX_NESTING_DEPTH, dropIntentForOffset } from './sectionTree'

const HEIGHT = 40

describe('dropIntentForOffset', () => {
  it('nests when the pointer is in the middle of a container row', () => {
    expect(dropIntentForOffset(HEIGHT / 2, HEIGHT, true)).toBe('inside')
  })

  it('drops as a sibling on the edges of a container row', () => {
    expect(dropIntentForOffset(1, HEIGHT, true)).toBe('before')
    expect(dropIntentForOffset(HEIGHT - 1, HEIGHT, true)).toBe('after')
  })

  it('puts the edge boundaries at the declared ratio', () => {
    const edge = HEIGHT * DROP_EDGE_RATIO
    expect(dropIntentForOffset(edge - 0.1, HEIGHT, true)).toBe('before')
    expect(dropIntentForOffset(edge + 0.1, HEIGHT, true)).toBe('inside')
    expect(dropIntentForOffset(HEIGHT - edge - 0.1, HEIGHT, true)).toBe('inside')
    expect(dropIntentForOffset(HEIGHT - edge + 0.1, HEIGHT, true)).toBe('after')
  })

  it('splits a non-container row in half, never nesting', () => {
    expect(dropIntentForOffset(HEIGHT / 2 - 1, HEIGHT, false)).toBe('before')
    expect(dropIntentForOffset(HEIGHT / 2 + 1, HEIGHT, false)).toBe('after')
    expect(dropIntentForOffset(HEIGHT / 2, HEIGHT, false)).toBe('after')
  })

  it('falls back to the row-wide intent when the row is unmeasurable', () => {
    // A zero-height row, or a pointer position the environment did not report,
    // must not resolve to a random edge.
    expect(dropIntentForOffset(0, 0, true)).toBe('inside')
    expect(dropIntentForOffset(0, 0, false)).toBe('after')
    expect(dropIntentForOffset(0, Number.NaN, true)).toBe('inside')
    expect(dropIntentForOffset(Number.NaN, HEIGHT, true)).toBe('inside')
    expect(dropIntentForOffset(Number.NaN, HEIGHT, false)).toBe('after')
  })
})

describe('MAX_NESTING_DEPTH', () => {
  it('leaves room for a realistic auth page', () => {
    expect(MAX_NESTING_DEPTH).toBeGreaterThanOrEqual(3)
  })
})
