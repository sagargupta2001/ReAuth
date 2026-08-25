import { describe, expect, it } from 'vitest'

import {
  DROP_EDGE_RATIO,
  DropAxis,
  axisForDirection,
  dropIntentForOffset,
  dropIntentForPoint,
} from './dropZones'

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

describe('axisForDirection', () => {
  it('reads a row parent as horizontal and everything else as vertical', () => {
    expect(axisForDirection('row')).toBe(DropAxis.Horizontal)
    expect(axisForDirection('column')).toBe(DropAxis.Vertical)
    expect(axisForDirection(undefined)).toBe(DropAxis.Vertical)
  })
})

describe('dropIntentForPoint', () => {
  const rect = { left: 100, top: 200, width: 80, height: 40 }

  it('uses the vertical edges inside a column parent', () => {
    const near = { clientX: 140, clientY: 202 }
    expect(dropIntentForPoint(near, rect, { acceptsChildren: true, axis: DropAxis.Vertical }))
      .toBe('before')
  })

  it('uses the horizontal edges inside a row parent', () => {
    // Same point: vertically near the top, horizontally in the middle. Reading
    // the wrong axis would call this "before" and point the indicator up.
    const point = { clientX: 140, clientY: 202 }
    expect(dropIntentForPoint(point, rect, { acceptsChildren: true, axis: DropAxis.Horizontal }))
      .toBe('inside')

    const left = { clientX: 102, clientY: 220 }
    expect(dropIntentForPoint(left, rect, { acceptsChildren: true, axis: DropAxis.Horizontal }))
      .toBe('before')
    const right = { clientX: 178, clientY: 220 }
    expect(dropIntentForPoint(right, rect, { acceptsChildren: true, axis: DropAxis.Horizontal }))
      .toBe('after')
  })

  it('splits a non-container in half along the parent axis', () => {
    expect(
      dropIntentForPoint({ clientX: 120, clientY: 220 }, rect, {
        acceptsChildren: false,
        axis: DropAxis.Horizontal,
      }),
    ).toBe('before')
    expect(
      dropIntentForPoint({ clientX: 170, clientY: 220 }, rect, {
        acceptsChildren: false,
        axis: DropAxis.Horizontal,
      }),
    ).toBe('after')
  })
})
