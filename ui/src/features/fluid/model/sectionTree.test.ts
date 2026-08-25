import { describe, expect, it } from 'vitest'

import { MAX_NESTING_DEPTH } from './sectionTree'

describe('MAX_NESTING_DEPTH', () => {
  it('leaves room for a realistic auth page', () => {
    expect(MAX_NESTING_DEPTH).toBeGreaterThanOrEqual(3)
  })
})
