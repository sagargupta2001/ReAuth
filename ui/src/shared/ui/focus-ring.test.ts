import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

/**
 * Repo-wide guard for one specific styling trap.
 *
 * Tailwind's `--tw-ring-offset-color` defaults to `#fff`, so any non-zero
 * `ring-offset-*` without an explicit offset colour paints a **white** ring —
 * invisible on a light theme, glaring on this dark one. It shipped that way on
 * the base `Input`, `Badge`, `ButtonGroup`, and two flow-builder nodes at once,
 * which is exactly the sort of thing a single assertion per component misses.
 */
const SRC_ROOT = join(process.cwd(), 'src')
const NONZERO_RING_OFFSET = /ring-offset-(?!background)(?!0\b)[a-z0-9[\]().-]+/g

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry)
    if (statSync(path).isDirectory()) return sourceFiles(path)
    return /\.tsx?$/.test(entry) && !/\.test\.tsx?$/.test(entry) ? [path] : []
  })
}

describe('focus ring offsets', () => {
  it('always pair a ring offset with an explicit offset colour', () => {
    const offenders = sourceFiles(SRC_ROOT)
      .map((path) => ({ path, source: readFileSync(path, 'utf8') }))
      .filter(({ source }) => {
        const widths = source.match(NONZERO_RING_OFFSET)
        if (!widths) return false
        return !source.includes('ring-offset-background')
      })
      .map(({ path }) => path.replace(`${process.cwd()}/`, ''))

    expect(offenders).toEqual([])
  })
})
