import { describe, expect, it } from 'vitest'

import {
  readLayoutShell,
  readTokenGroup,
  readTokenString,
  withLayoutShell,
  withTokenValue,
} from './tokenAccess'
import { DEFAULT_LAYOUT_SHELL, LayoutShell } from '@/features/fluid/model/layoutShells'
import { ColorToken, TokenGroup } from '@/features/fluid/model/tokens'

describe('readTokenGroup', () => {
  it('returns the nested record when present', () => {
    const tokens = { colors: { primary: '#fff' } }
    expect(readTokenGroup(tokens, TokenGroup.Colors)).toEqual({ primary: '#fff' })
  })

  it('returns an empty record for missing, array, or primitive values', () => {
    expect(readTokenGroup({}, TokenGroup.Colors)).toEqual({})
    expect(readTokenGroup({ colors: ['#fff'] }, TokenGroup.Colors)).toEqual({})
    expect(readTokenGroup({ colors: 'nope' }, TokenGroup.Colors)).toEqual({})
    expect(readTokenGroup({ colors: null }, TokenGroup.Colors)).toEqual({})
  })
})

describe('readTokenString', () => {
  it('reads strings and coerces numbers', () => {
    const tokens = { colors: { primary: '#111827' }, radius: { base: 8 } }
    expect(readTokenString(tokens, TokenGroup.Colors, ColorToken.Primary)).toBe('#111827')
    expect(readTokenString(tokens, TokenGroup.Radius, 'base')).toBe('8')
  })

  it('falls back when the token is missing or not a scalar', () => {
    expect(readTokenString({}, TokenGroup.Colors, ColorToken.Primary)).toBe('')
    expect(readTokenString({}, TokenGroup.Colors, ColorToken.Primary, '#000')).toBe('#000')
    expect(
      readTokenString({ colors: { primary: { nested: true } } }, TokenGroup.Colors, 'primary', 'x'),
    ).toBe('x')
  })
})

describe('withTokenValue', () => {
  it('replaces one token without touching siblings', () => {
    const tokens = {
      colors: { primary: '#111827', background: '#fff' },
      typography: { font_family: 'Inter' },
    }

    const next = withTokenValue(tokens, TokenGroup.Colors, ColorToken.Primary, '#ff0000')

    expect(next).toEqual({
      colors: { primary: '#ff0000', background: '#fff' },
      typography: { font_family: 'Inter' },
    })
    expect(next).not.toBe(tokens)
    expect(tokens.colors.primary).toBe('#111827')
  })

  it('creates the group when it does not exist yet', () => {
    expect(withTokenValue({}, TokenGroup.Radius, 'base', '12px')).toEqual({
      radius: { base: '12px' },
    })
  })
})

describe('layout helpers', () => {
  it('reads the shell with a default', () => {
    expect(readLayoutShell({ shell: LayoutShell.Minimal })).toBe(LayoutShell.Minimal)
    expect(readLayoutShell({})).toBe(DEFAULT_LAYOUT_SHELL)
    expect(readLayoutShell({ shell: 42 })).toBe(DEFAULT_LAYOUT_SHELL)
  })

  it('normalizes slots when writing the shell', () => {
    expect(withLayoutShell({}, LayoutShell.SplitScreen)).toEqual({
      shell: LayoutShell.SplitScreen,
      slots: ['main'],
    })
    expect(withLayoutShell({ slots: ['a', 'b'] }, LayoutShell.Minimal)).toEqual({
      shell: LayoutShell.Minimal,
      slots: ['a', 'b'],
    })
  })

  it('preserves unrelated layout keys', () => {
    expect(withLayoutShell({ custom: true }, LayoutShell.CenteredCard)).toEqual({
      custom: true,
      shell: LayoutShell.CenteredCard,
      slots: ['main'],
    })
  })
})
