import { describe, expect, it } from 'vitest'

import { evaluateContrastPair, resolveColorRef } from './contrastPairs'
import { ColorToken, TOKEN_FALLBACK, TokenGroup } from '@/features/fluid/model/tokens'

const textRef = {
  group: TokenGroup.Colors,
  token: ColorToken.Text,
  fallback: TOKEN_FALLBACK.text,
} as const
const backgroundRef = {
  group: TokenGroup.Colors,
  token: ColorToken.Background,
  fallback: TOKEN_FALLBACK.background,
} as const

describe('resolveColorRef', () => {
  it('reads a token when set', () => {
    expect(resolveColorRef({ colors: { text: '#123456' } }, textRef)).toBe('#123456')
  })

  it('falls back when the token is empty', () => {
    expect(resolveColorRef({}, textRef)).toBe(TOKEN_FALLBACK.text)
    expect(resolveColorRef({ colors: { text: '' } }, textRef)).toBe(TOKEN_FALLBACK.text)
  })

  it('passes literals through', () => {
    expect(resolveColorRef({}, { literal: '#ffffff' })).toBe('#ffffff')
  })
})

describe('evaluateContrastPair', () => {
  const pair = {
    label: 'Text on background',
    foreground: textRef,
    background: backgroundRef,
    minRatio: 4.5,
  }

  it('passes a high-contrast pair', () => {
    const result = evaluateContrastPair(
      { colors: { text: '#000000', background: '#ffffff' } },
      pair,
    )
    expect(result.ratio).toBeCloseTo(21, 0)
    expect(result.passes).toBe(true)
  })

  it('fails a low-contrast pair', () => {
    const result = evaluateContrastPair(
      { colors: { text: '#777777', background: '#808080' } },
      pair,
    )
    expect(result.passes).toBe(false)
    expect(result.ratio).not.toBeNull()
  })

  it('honours a lower minimum for UI-sized pairs', () => {
    const tokens = { colors: { text: '#767676', background: '#ffffff' } }
    expect(evaluateContrastPair(tokens, { ...pair, minRatio: 4.5 }).passes).toBe(true)
    expect(evaluateContrastPair(tokens, { ...pair, minRatio: 7 }).passes).toBe(false)
  })

  it('does not report an unmeasurable pair as a failure', () => {
    const result = evaluateContrastPair(
      { colors: { text: 'not-a-color', background: '#ffffff' } },
      pair,
    )
    expect(result.ratio).toBeNull()
    expect(result.passes).toBe(true)
  })
})
