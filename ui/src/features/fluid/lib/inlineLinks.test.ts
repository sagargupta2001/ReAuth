import { describe, expect, it } from 'vitest'

import { parseInlineLinks } from './inlineLinks'

describe('parseInlineLinks', () => {
  it('splits copy around a link', () => {
    expect(parseInlineLinks('I accept the [Terms](/terms) today')).toEqual([
      { kind: 'text', text: 'I accept the ' },
      { kind: 'link', text: 'Terms', href: '/terms' },
      { kind: 'text', text: ' today' },
    ])
  })

  it('handles several links in one line', () => {
    const segments = parseInlineLinks('[A](/a) and [B](/b)')
    expect(segments.filter((segment) => segment.kind === 'link')).toHaveLength(2)
    // The string opens with a link, so there is no leading text segment.
    expect(segments[0]).toEqual({ kind: 'link', text: 'A', href: '/a' })
    expect(segments[1]).toEqual({ kind: 'text', text: ' and ' })
  })

  it('leaves plain copy as one segment', () => {
    expect(parseInlineLinks('no links here')).toEqual([
      { kind: 'text', text: 'no links here' },
    ])
  })

  it('keeps malformed markup literal rather than swallowing it', () => {
    // Dropping text a builder typed is worse than showing the brackets.
    expect(parseInlineLinks('[unclosed(/x)')).toEqual([
      { kind: 'text', text: '[unclosed(/x)' },
    ])
  })

  it('is empty for nothing', () => {
    expect(parseInlineLinks(undefined)).toEqual([])
  })
})
