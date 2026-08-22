import { describe, expect, it } from 'vitest'

import { queryKeys } from './queryKeys'

/** Mirrors how React Query decides whether a filter key matches a query key. */
function isPrefixOf(filter: readonly unknown[], target: readonly unknown[]): boolean {
  return filter.every((segment, index) => segment === target[index])
}

describe('themePreview keys', () => {
  it('makes the short form a prefix of the fully-specified form', () => {
    const filter = queryKeys.themePreview('master', 'theme-1')
    const live = queryKeys.themePreview('master', 'theme-1', 'login')

    // Eight mutation hooks invalidate with the short form. With trailing
    // `undefined` segments it matched nothing, so the preview never refreshed
    // after publish, rollback, or start-draft-from-version.
    expect(isPrefixOf(filter, live)).toBe(true)
  })

  it('does not leave trailing undefined segments', () => {
    expect(queryKeys.themePreview('master', 'theme-1')).toEqual([
      'theme-preview',
      'master',
      'theme-1',
    ])
  })

  it('keeps inner positions so a later-only argument cannot collide', () => {
    const nodeOnly = queryKeys.themePreview('master', 'theme-1', undefined, 'node-1')
    const pageOnly = queryKeys.themePreview('master', 'theme-1', 'node-1')
    expect(nodeOnly).not.toEqual(pageOnly)
  })

  it('still distinguishes different pages', () => {
    const login = queryKeys.themePreview('master', 'theme-1', 'login')
    const register = queryKeys.themePreview('master', 'theme-1', 'register')
    expect(login).not.toEqual(register)
    expect(isPrefixOf(queryKeys.themePreview('master', 'theme-1'), register)).toBe(true)
  })

  it('does not match a different theme', () => {
    const filter = queryKeys.themePreview('master', 'theme-1')
    const other = queryKeys.themePreview('master', 'theme-2', 'login')
    expect(isPrefixOf(filter, other)).toBe(false)
  })
})

describe('themeSnapshot keys', () => {
  it('makes the realm-only form a prefix of a specified one', () => {
    const filter = queryKeys.themeSnapshot('master')
    const live = queryKeys.themeSnapshot('master', { pageKey: 'login' })
    expect(isPrefixOf(filter, live)).toBe(true)
  })

  it('drops trailing undefined params', () => {
    expect(queryKeys.themeSnapshot('master')).toEqual(['theme-snapshot', 'master'])
  })
})
