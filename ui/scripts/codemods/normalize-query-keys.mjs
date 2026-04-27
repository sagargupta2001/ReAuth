import { promises as fs } from 'node:fs'
import path from 'node:path'

const ROOT = path.resolve(process.cwd(), 'src')
const QUERY_KEYS_IMPORT = "@/shared/lib/queryKeys"

const simpleMap = new Map([
  ['setup-status', 'setupStatus'],
  ['user', 'user'],
  ['users', 'users'],
  ['client', 'client'],
  ['clients', 'clients'],
  ['role', 'role'],
  ['roles', 'roles'],
  ['group', 'group'],
  ['groups', 'groups'],
  ['group-children', 'groupChildren'],
  ['group-members', 'groupMembers'],
  ['group-member-list', 'groupMemberList'],
  ['group-roles', 'groupRoles'],
  ['group-role-list', 'groupRoleList'],
  ['group-delete-summary', 'groupDeleteSummary'],
  ['sessions', 'sessions'],
  ['user-roles', 'userRoles'],
  ['user-role-list', 'userRoleList'],
  ['role-members', 'roleMembers'],
  ['role-member-list', 'roleMemberList'],
  ['role-composites', 'roleComposites'],
  ['role-composite-list', 'roleCompositeList'],
  ['role-permissions', 'rolePermissions'],
  ['permissions-definitions', 'permissionsDefinitions'],
  ['flows', 'flows'],
  ['flow', 'flow'],
  ['flow-draft', 'flowDraft'],
  ['flow-drafts', 'flowDrafts'],
  ['flow-versions', 'flowVersions'],
  ['flow-nodes', 'flowNodes'],
  ['harbor-jobs', 'harborJobs'],
  ['harbor-job-details', 'harborJobDetails'],
  ['observability-logs', 'observabilityLogs'],
  ['observability-traces', 'observabilityTraces'],
  ['observability-trace-spans', 'observabilityTraceSpans'],
  ['observability-metrics', 'observabilityMetrics'],
  ['observability-cache-stats', 'observabilityCacheStats'],
  ['observability-log-targets', 'observabilityLogTargets'],
  ['event-routing-metrics', 'eventRoutingMetrics'],
  ['themes', 'themes'],
  ['theme-versions', 'themeVersions'],
  ['theme-version-snapshot', 'themeVersionSnapshot'],
  ['theme-template-gaps', 'themeTemplateGaps'],
  ['theme-assets', 'themeAssets'],
  ['theme-draft', 'themeDraft'],
  ['theme-preview', 'themePreview'],
  ['theme-snapshot', 'themeSnapshot'],
  ['theme-pages', 'themePages'],
  ['theme-bindings', 'themeBindings'],
  ['active-theme', 'activeTheme'],
  ['realms', 'realms'],
  ['realm', 'realm'],
  ['realm-bindings', 'realmBindings'],
  ['realm-email-settings', 'realmEmailSettings'],
  ['realm-recovery-settings', 'realmRecoverySettings'],
  ['realm-security-headers', 'realmSecurityHeaders'],
  ['omni-search', 'omniSearch'],
  ['webhooks', 'webhooks'],
  ['webhooks-by-id', 'webhooksById'],
  ['webhook-deliveries', 'webhookDeliveries'],
  ['delivery-logs', 'webhookDeliveryLogs'],
])

const SPECIAL_KEYS = new Set(['theme-bindings', 'webhooks'])

const stringLiteral = (value) => value.replace(/^['"`]|['"`]$/g, '')
const isStringLiteral = (value) => /^['"`].*['"`]$/.test(value.trim())

function splitTopLevel(input) {
  const parts = []
  let current = ''
  let depth = 0
  let inString = false
  let stringChar = ''
  let escaped = false

  for (let i = 0; i < input.length; i += 1) {
    const char = input[i]
    if (escaped) {
      current += char
      escaped = false
      continue
    }
    if (char === '\\\\') {
      current += char
      escaped = true
      continue
    }
    if (inString) {
      current += char
      if (char === stringChar) {
        inString = false
      }
      continue
    }
    if (char === '"' || char === "'" || char === '`') {
      inString = true
      stringChar = char
      current += char
      continue
    }
    if (char === '[' || char === '{' || char === '(') {
      depth += 1
      current += char
      continue
    }
    if (char === ']' || char === '}' || char === ')') {
      depth -= 1
      current += char
      continue
    }
    if (char === ',' && depth === 0) {
      parts.push(current.trim())
      current = ''
      continue
    }
    current += char
  }
  if (current.trim()) {
    parts.push(current.trim())
  }
  return parts
}

function findMatchingBracket(text, startIndex) {
  let depth = 0
  let inString = false
  let stringChar = ''
  let escaped = false

  for (let i = startIndex; i < text.length; i += 1) {
    const char = text[i]
    if (escaped) {
      escaped = false
      continue
    }
    if (char === '\\\\') {
      escaped = true
      continue
    }
    if (inString) {
      if (char === stringChar) {
        inString = false
      }
      continue
    }
    if (char === '"' || char === "'" || char === '`') {
      inString = true
      stringChar = char
      continue
    }
    if (char === '[') {
      depth += 1
      continue
    }
    if (char === ']') {
      depth -= 1
      if (depth === 0) {
        return i
      }
    }
  }
  return -1
}

function buildReplacement(elements) {
  if (!elements.length) return null
  const first = elements[0]
  if (!isStringLiteral(first)) return null
  const key = stringLiteral(first.trim())
  const args = elements.slice(1).filter(Boolean)

  if (SPECIAL_KEYS.has(key)) {
    if (key === 'theme-bindings') {
      const marker = args[0]
      if (isStringLiteral(marker) && stringLiteral(marker) === 'client') {
        const realm = args[1]
        const clientId = args[2]
        if (!realm || !clientId) return null
        return `queryKeys.themeBindingClient(${realm}, ${clientId})`
      }
      return `queryKeys.themeBindings(${args.join(', ')})`
    }

    if (key === 'webhooks') {
      if (args.length === 1) {
        return `queryKeys.webhooks(${args[0]})`
      }
      if (args.length >= 2) {
        const candidate = args[1]
        const looksLikeParams =
          candidate.includes('{') || candidate.includes('}') || candidate.includes('params')
        if (looksLikeParams) {
          return `queryKeys.webhooks(${args[0]}, ${candidate})`
        }
        return `queryKeys.webhooksById(${args[0]}, ${candidate})`
      }
    }
  }

  const fn = simpleMap.get(key)
  if (!fn) return null
  if (args.length === 0) {
    return `queryKeys.${fn}()`
  }
  return `queryKeys.${fn}(${args.join(', ')})`
}

function ensureQueryKeysImport(source) {
  const importRegex = new RegExp(
    `import\\s*\\{([^}]+)\\}\\s*from\\s*['"]${QUERY_KEYS_IMPORT}['"]`,
  )
  const match = source.match(importRegex)
  if (match) {
    if (match[1].includes('queryKeys')) return source
    const updated = match[0].replace(match[1], `${match[1].trim()}, queryKeys`)
    return source.replace(match[0], updated)
  }

  const lines = source.split('\n')
  let insertAt = 0
  while (insertAt < lines.length) {
    const line = lines[insertAt]
    if (line.startsWith('import ')) {
      insertAt += 1
      continue
    }
    if (line.trim() === '') {
      insertAt += 1
      continue
    }
    break
  }

  lines.splice(insertAt, 0, `import { queryKeys } from '${QUERY_KEYS_IMPORT}'`)
  return lines.join('\n')
}

async function collectFiles(dir) {
  const entries = await fs.readdir(dir, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...(await collectFiles(fullPath)))
    } else if (entry.isFile() && (fullPath.endsWith('.ts') || fullPath.endsWith('.tsx'))) {
      files.push(fullPath)
    }
  }
  return files
}

async function run() {
  const files = await collectFiles(ROOT)
  let changedFiles = 0
  let replacements = 0
  const warnings = []

  for (const file of files) {
    const original = await fs.readFile(file, 'utf8')
    if (!original.includes('queryKey: [')) continue

    let updated = ''
    let lastIndex = 0
    let changed = false
    const regex = /queryKey\s*:\s*\[/g
    let match

    while ((match = regex.exec(original)) !== null) {
      const arrayStart = original.indexOf('[', match.index)
      const arrayEnd = findMatchingBracket(original, arrayStart)
      if (arrayStart === -1 || arrayEnd === -1) continue

      const arrayBody = original.slice(arrayStart + 1, arrayEnd)
      const elements = splitTopLevel(arrayBody)
      const replacement = buildReplacement(elements)
      if (!replacement) {
        warnings.push(`Skipped ${file}: ${arrayBody.trim()}`)
        continue
      }

      updated += original.slice(lastIndex, arrayStart) + replacement
      lastIndex = arrayEnd + 1
      changed = true
      replacements += 1
    }

    if (!changed) continue
    updated += original.slice(lastIndex)
    updated = ensureQueryKeysImport(updated)

    if (updated !== original) {
      await fs.writeFile(file, updated, 'utf8')
      changedFiles += 1
    }
  }

  console.log(`Updated ${changedFiles} files with ${replacements} replacements.`)
  if (warnings.length > 0) {
    console.log('Skipped entries:')
    warnings.forEach((warning) => console.log(`- ${warning}`))
  }
}

run().catch((err) => {
  console.error(err)
  process.exit(1)
})
