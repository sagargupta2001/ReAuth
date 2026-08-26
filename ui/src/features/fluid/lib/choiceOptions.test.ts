import { describe, expect, it } from 'vitest'

import { parseChoices } from './choiceOptions'

describe('parseChoices', () => {
  it('reads one option per line', () => {
    expect(parseChoices('red\ngreen')).toEqual([
      { value: 'red', label: 'red' },
      { value: 'green', label: 'green' },
    ])
  })

  it('splits a value from its label', () => {
    expect(parseChoices('gb|United Kingdom')).toEqual([
      { value: 'gb', label: 'United Kingdom' },
    ])
  })

  it('falls back to the value when the label is blank', () => {
    expect(parseChoices('gb|  ')).toEqual([{ value: 'gb', label: 'gb' }])
  })

  it('skips blank lines rather than making empty options', () => {
    // A textarea hands you a trailing newline for free.
    expect(parseChoices('a\n\n b \n')).toEqual([
      { value: 'a', label: 'a' },
      { value: 'b', label: 'b' },
    ])
  })

  it('keeps a label containing further separators intact', () => {
    expect(parseChoices('k|a|b')).toEqual([{ value: 'k', label: 'a|b' }])
  })

  it('accepts a structured list from a hand-authored blueprint', () => {
    expect(parseChoices([{ value: 'a', label: 'A' }, 'b'])).toEqual([
      { value: 'a', label: 'A' },
      { value: 'b', label: 'b' },
    ])
  })

  it('is empty for nothing', () => {
    expect(parseChoices(undefined)).toEqual([])
    expect(parseChoices('')).toEqual([])
  })
})
