import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

import type { ThemeNode } from '@/entities/theme/model/types'
import { EXPANDED_COMPONENTS } from '@/features/fluid/lib/componentRegistry'
import { FLUID_BLOCKS, type FluidBlockDefinition } from '@/features/fluid/model/blockCatalog'
import {
  FieldTarget,
  InspectorFieldKind,
  matchesNode,
  type InspectorField,
  type InspectorSection,
} from '@/features/fluid/model/inspectorFields'
import { INSPECTOR_SECTIONS } from '@/features/fluid/model/inspectorSchema'
import { SettingsFieldKind } from '@/features/fluid/model/settingsFields'
import { THEME_SETTINGS_SECTIONS } from '@/features/fluid/model/themeSettingsSchema'

/**
 * Generates `docs/memory/22-fluid-capability-matrix.md`.
 *
 * The inventory is derived from the schemas rather than written by hand, so it
 * cannot drift: adding a block, an inspector field, or a token changes this
 * file's output, and the snapshot fails until the doc is regenerated with
 * `npm run test -- -u`.
 *
 * It also asserts the two invariants the matrix exists to protect — every block
 * is reachable from the inspector, and every block renders in *both* renderers.
 */

const HERE = dirname(fileURLToPath(import.meta.url))
const REPO_ROOT = resolve(HERE, '../../../../..')
const DOC_PATH = resolve(REPO_ROOT, 'docs/memory/22-fluid-capability-matrix.md')

const RENDERER_SOURCES = {
  FluidCanvas: 'ui/src/features/fluid/components/FluidCanvas.tsx',
  FluidLoginScreen: 'ui/src/features/auth/screens/FluidLoginScreen.tsx',
} as const

/** Files that consume `node.props` on behalf of one or both renderers. */
const PROP_CONSUMERS = [
  ...Object.values(RENDERER_SOURCES),
  'ui/src/features/fluid/lib/nodeVisuals.ts',
  'ui/src/features/fluid/lib/componentRegistry.ts',
]

/**
 * Props read by a renderer that no schema field writes, with the reason. An
 * entry here is a deliberate exception; anything else shows up as a gap.
 */
const UNEXPOSED_BY_DESIGN: Record<string, string> = {
  visible: 'Toggled by the Input slots panel, not by a schema field.',
  visible_if: 'Authored in the blueprint; binds the block to auth context.',
  width: 'Legacy pre-`size` fallback, still read for old blueprints.',
  height: 'Legacy pre-`size` fallback, still read for old blueprints.',
  width_value: 'Legacy pre-`size` fallback, still read for old blueprints.',
  height_value: 'Legacy pre-`size` fallback, still read for old blueprints.',
}

/** Render targets the block catalog can insert; anything else is slot-only. */
const CATALOG_RENDER_TARGETS = new Set(
  FLUID_BLOCKS.map((block) => block.node.component ?? block.node.type),
)

function readSource(relativePath: string): string {
  return readFileSync(resolve(REPO_ROOT, relativePath), 'utf8')
}

/** `props.foo` and `props['foo']` occurrences in a source file. */
function propsReadBy(source: string): Set<string> {
  const found = new Set<string>()
  for (const match of source.matchAll(/props\.([a-z][a-z0-9_]*)/g)) found.add(match[1])
  for (const match of source.matchAll(/props\['([a-z][a-z0-9_]*)'\]/g)) found.add(match[1])
  return found
}

/** A representative node for a block, for `matchesNode`. */
function sampleNodeFor(block: FluidBlockDefinition): ThemeNode {
  return {
    id: `sample-${block.id}`,
    type: block.node.type,
    component: block.node.component,
    props: block.node.props ?? {},
  }
}

function renderTargetOf(block: FluidBlockDefinition): string {
  return block.node.component ?? block.node.type
}

function sectionsFor(block: FluidBlockDefinition): InspectorSection[] {
  const node = sampleNodeFor(block)
  return INSPECTOR_SECTIONS.filter((section) => matchesNode(section.appliesTo, node))
}

/** Where a field writes, as a `target.key` path. */
function writeTargetOf(field: InspectorField): string {
  switch (field.kind) {
    case InspectorFieldKind.Readonly:
    case InspectorFieldKind.Custom:
      return '—'
    case InspectorFieldKind.PaddingBox:
      return '`layout.padding`'
    case InspectorFieldKind.Dimension:
      return `\`size.${field.axis}\` + \`size.${field.axis}_value\``
    default:
      return `\`${field.target}.${field.key}\``
  }
}

function fieldLabelOf(field: InspectorField): string {
  return field.kind === InspectorFieldKind.Custom ? `${field.panel} panel` : field.label
}

function controlOf(field: InspectorField): string {
  return field.kind
}

/** Props an inspector field writes into `node.props`. */
function propsWrittenByInspector(): Set<string> {
  const written = new Set<string>()
  for (const section of INSPECTOR_SECTIONS) {
    for (const field of section.fields) {
      if (
        field.kind !== InspectorFieldKind.Readonly &&
        field.kind !== InspectorFieldKind.Custom &&
        field.kind !== InspectorFieldKind.PaddingBox &&
        field.kind !== InspectorFieldKind.Dimension &&
        field.target === FieldTarget.Props
      ) {
        written.add(field.key)
      }
    }
  }
  return written
}

function appliesToLabel(section: InspectorSection): string {
  if (!section.appliesTo) return 'every block'
  const parts = [...(section.appliesTo.types ?? []), ...(section.appliesTo.components ?? [])]
  return parts
    .map((part) => (CATALOG_RENDER_TARGETS.has(part) ? part : `${part} (slot-only)`))
    .join(', ')
}

function themeFieldToken(field: (typeof THEME_SETTINGS_SECTIONS)[number]['fields'][number]) {
  switch (field.kind) {
    case SettingsFieldKind.LayoutShell:
      return '`layout.shell`'
    case SettingsFieldKind.Assets:
      return '— (theme assets)'
    case SettingsFieldKind.ColorContrast:
      return '— (read-only report)'
    default:
      return `\`tokens.${field.group}.${field.token}\``
  }
}

function themeFieldLabel(field: (typeof THEME_SETTINGS_SECTIONS)[number]['fields'][number]) {
  if ('label' in field && field.label) return field.label
  // Kinds that are a whole panel rather than one labelled value carry only an id.
  return field.id
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

function table(headers: string[], rows: string[][]): string {
  const lines = [
    `| ${headers.join(' | ')} |`,
    `|${headers.map(() => '---').join('|')}|`,
    ...rows.map((row) => `| ${row.join(' | ')} |`),
  ]
  return lines.join('\n')
}

function buildMatrix(): string {
  const sources = Object.fromEntries(
    Object.entries(RENDERER_SOURCES).map(([name, path]) => [name, readSource(path)]),
  ) as Record<keyof typeof RENDERER_SOURCES, string>

  const rendererProps = new Set<string>()
  for (const path of PROP_CONSUMERS) {
    for (const prop of propsReadBy(readSource(path))) rendererProps.add(prop)
  }
  const inspectorProps = propsWrittenByInspector()

  const out: string[] = []

  out.push('# Fluid Capability Matrix')
  out.push('')
  out.push(
    '> Generated from the Fluid schemas by',
    '> `ui/src/features/fluid/model/capabilityMatrix.test.ts`. Do not edit by hand —',
    '> run `cd ui && npm run test -- -u` after changing `blockCatalog.ts`,',
    '> `inspectorSchema.ts`, `themeSettingsSchema.ts`, or either renderer.',
  )
  out.push('')
  out.push(
    'This is the inventory of what the Fluid builder can currently express: which',
    'blocks exist, what can be styled on each of them, and which global theme',
    'options a page inherits. Use it to see what is already there before adding a',
    'control, and to see the gaps worth closing next. The design rules behind these',
    'lists live in `18-fluid-theme-builder.md` §5.4.',
  )
  out.push('')

  out.push('## 1. Blocks')
  out.push('')
  out.push(
    table(
      ['Block', 'Catalog id', 'Renders as', 'Category', 'Children', 'Inspector sections'],
      FLUID_BLOCKS.map((block) => [
        block.label,
        `\`${block.id}\``,
        `\`${block.node.type}\`${block.node.component ? ` / \`${block.node.component}\`` : ''}`,
        block.category,
        block.acceptsChildren ? 'yes' : 'no',
        sectionsFor(block)
          .map((section) => section.title)
          .join(', '),
      ]),
    ),
  )
  out.push('')
  out.push(
    'Adding a block: an id in `FluidBlockId`, a definition in `FLUID_BLOCKS`, a',
    'preview in `BLOCK_PREVIEWS` (compile-enforced), and — unless the registry',
    'expands it — a `case` in **both** renderers.',
  )
  out.push('')

  out.push('## 2. Per-block styling options')
  out.push('')
  out.push(
    'One subsection per inspector section, in the order the inspector renders them.',
    'A field names the exact path it writes, because `props.align` (text) and',
    '`layout.align` (flex cross-axis) are different properties.',
  )
  out.push('')
  for (const section of INSPECTOR_SECTIONS) {
    out.push(`### ${section.title}`)
    out.push('')
    out.push(`Applies to: ${appliesToLabel(section)}.`)
    out.push('')
    out.push(
      table(
        ['Field', 'Writes', 'Control'],
        section.fields.map((field) => [
          fieldLabelOf(field),
          writeTargetOf(field),
          `\`${controlOf(field)}\``,
        ]),
      ),
    )
    out.push('')
  }

  out.push('## 3. Global theme options')
  out.push('')
  out.push('Theme-wide tokens, edited in the theme settings panel and inherited by every page.')
  out.push('')
  for (const section of THEME_SETTINGS_SECTIONS) {
    out.push(`### ${section.title}`)
    out.push('')
    out.push(
      table(
        ['Field', 'Writes', 'Control'],
        section.fields.map((field) => [
          themeFieldLabel(field),
          themeFieldToken(field),
          `\`${field.kind}\``,
        ]),
      ),
    )
    out.push('')
  }

  out.push('## 4. Renderer coverage')
  out.push('')
  out.push(
    '`FluidCanvas` is the builder preview and `FluidLoginScreen` is what users get.',
    'A block handled by only one of them looks fine in review and is broken in',
    'production, or the reverse — which is exactly how `ProviderButtons` shipped.',
  )
  out.push('')
  out.push(
    table(
      ['Block', 'Handled by', 'FluidCanvas', 'FluidLoginScreen'],
      FLUID_BLOCKS.map((block) => {
        const target = renderTargetOf(block)
        const expanded = EXPANDED_COMPONENTS.includes(target)
        const handledBy = expanded ? '`componentRegistry` expansion' : 'inline `case`'
        const check = (source: string) =>
          expanded || rendererHandles(source, block) ? 'yes' : '**no**'
        return [
          block.label,
          handledBy,
          check(sources.FluidCanvas),
          check(sources.FluidLoginScreen),
        ]
      }),
    ),
  )
  out.push('')

  out.push('## 5. Derived gaps')
  out.push('')

  const writtenButUnread = [...inspectorProps].filter((prop) => !rendererProps.has(prop)).sort()
  out.push('### Controls whose value no renderer reads')
  out.push('')
  if (writtenButUnread.length === 0) {
    out.push('None. Every inspector field writes a prop a renderer consumes.')
  } else {
    out.push(
      'Each of these is a control that appears to work and changes nothing on screen.',
      '',
      ...writtenButUnread.map((prop) => `- \`props.${prop}\``),
    )
  }
  out.push('')

  const readButUnexposed = [...rendererProps].filter((prop) => !inspectorProps.has(prop)).sort()
  out.push('### Props a renderer reads with no inspector control')
  out.push('')
  out.push(
    'Capability that already renders but can only be reached by hand-editing a',
    'blueprint. Annotated entries are deliberate; the rest are candidates for a new',
    'control.',
  )
  out.push('')
  out.push(
    table(
      ['Prop', 'Status'],
      readButUnexposed.map((prop) => [
        `\`props.${prop}\``,
        UNEXPOSED_BY_DESIGN[prop] ?? '**Unexposed — candidate for a control.**',
      ]),
    ),
  )
  out.push('')
  out.push(
    'Heuristic: props are collected by scanning `props.<key>` in the renderers,',
    '`nodeVisuals.ts`, and `componentRegistry.ts`. A prop read through a computed',
    'key would not appear here.',
  )
  out.push('')

  return `${out.join('\n')}`
}

/** Whether a renderer has an inline branch for this block. */
function rendererHandles(source: string, block: FluidBlockDefinition): boolean {
  if (block.node.component) {
    return source.includes(`=== '${block.node.component.toLowerCase()}'`)
  }
  return source.includes(`case '${block.node.type}'`)
}

describe('fluid capability matrix', () => {
  it('matches the generated inventory doc', async () => {
    await expect(buildMatrix()).toMatchFileSnapshot(DOC_PATH)
  })

  it('renders every block in both the builder preview and the runtime', () => {
    const canvas = readSource(RENDERER_SOURCES.FluidCanvas)
    const runtime = readSource(RENDERER_SOURCES.FluidLoginScreen)

    const unhandled = FLUID_BLOCKS.filter((block) => {
      if (EXPANDED_COMPONENTS.includes(renderTargetOf(block))) return false
      return !rendererHandles(canvas, block) || !rendererHandles(runtime, block)
    }).map((block) => block.label)

    expect(unhandled).toEqual([])
  })

  it('keeps every unexposed-prop annotation pointing at a real gap', () => {
    // A stale annotation would silently hide a prop that has since gained a
    // control, or explain away one that no longer exists.
    const rendererProps = new Set<string>()
    for (const path of PROP_CONSUMERS) {
      for (const prop of propsReadBy(readSource(path))) rendererProps.add(prop)
    }
    const inspectorProps = propsWrittenByInspector()
    const stale = Object.keys(UNEXPOSED_BY_DESIGN).filter(
      (prop) => !rendererProps.has(prop) || inspectorProps.has(prop),
    )
    expect(stale).toEqual([])
  })

  it('gives every block at least one inspector section beyond the type readout', () => {
    const unreachable = FLUID_BLOCKS.filter((block) => sectionsFor(block).length <= 1).map(
      (block) => block.label,
    )
    expect(unreachable).toEqual([])
  })
})
