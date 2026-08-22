import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

import { describe, expect, it } from 'vitest'

import { TAB_GROUPS } from '@/features/breadcrumb/config/breadcrumb-config'

const SRC = join(process.cwd(), 'src')

/**
 * Adding a tabbed detail page means touching two places: the `:tab?` route and
 * the breadcrumb's `TAB_GROUPS`. The theme page shipped with neither, and the
 * missing breadcrumb entry is invisible until someone opens the dropdown.
 */
function routedTabPaths(): string[] {
  const source = readFileSync(join(SRC, 'app/routerConfig.tsx'), 'utf8')
  return [...source.matchAll(/path:\s*'([^']*:tab\?[^']*)'/g)].map((match) => match[1])
}

/** The owning section: the segment just before the entity id. */
function sectionOf(path: string): string {
  const segments = path.split('/').filter(Boolean)
  const tabIndex = segments.indexOf(':tab?')
  return segments[tabIndex - 2] ?? ''
}

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry)
    if (statSync(path).isDirectory()) return sourceFiles(path)
    return /\.tsx?$/.test(entry) && !/\.test\.tsx?$/.test(entry) ? [path] : []
  })
}

describe('routed tabs', () => {
  it('finds the tabbed detail routes', () => {
    // Guards the regex itself: a silent zero-match would make the rest vacuous.
    expect(routedTabPaths().length).toBeGreaterThanOrEqual(7)
  })

  it('gives every tabbed route a breadcrumb tab group', () => {
    const missing = routedTabPaths()
      .map((path) => ({ path, section: sectionOf(path) }))
      .filter(({ section }) => !TAB_GROUPS[section])

    expect(missing).toEqual([])
  })

  it('keeps the tab logic in useRoutedTab rather than per page', () => {
    // The param/validate/redirect block was copy-pasted into seven pages.
    const offenders = sourceFiles(SRC)
      .filter((path) => /[/\\](pages|widgets)[/\\]/.test(path))
      .filter((path) => {
        const source = readFileSync(path, 'utf8')
        return /\bvalidTabs\b|\bhandleTabChange\b/.test(source)
      })
      .map((path) => path.replace(`${process.cwd()}/`, ''))

    expect(offenders).toEqual([])
  })
})
