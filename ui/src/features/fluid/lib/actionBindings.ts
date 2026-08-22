/**
 * Action-binding helpers for the inspector's Actions tab.
 *
 * Kept out of the component so the payload-path rules are unit-testable — they
 * are the part a builder is most likely to get wrong, and a bad path silently
 * sends nothing at runtime.
 */

export interface InspectorAction {
  action_id?: string
  trigger?: string
  signal?: {
    type?: string
    node_id?: string
    payload_map?: Record<string, unknown>
  }
}

const RECENT_ACTION_NODE_KEY = 'reauth.fluid.action-node-ids'
const MAX_RECENT_ACTION_NODES = 20

/** Prefixes a payload path may start with. */
export const PAYLOAD_PATH_ROOTS = ['inputs.', 'context.'] as const

export function readRecentActionNodeIds(): string[] {
  if (typeof window === 'undefined') return []
  try {
    const raw = window.localStorage.getItem(RECENT_ACTION_NODE_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as string[]
    return Array.isArray(parsed) ? parsed : []
  } catch {
    // A corrupt entry must not break the inspector; recent ids are a convenience.
    return []
  }
}

/** Stores the most recent node ids, newest first, capped. */
export function writeRecentActionNodeIds(entries: string[]): void {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(RECENT_ACTION_NODE_KEY, JSON.stringify(entries))
}

/** Merges a node id into the recent list, newest first and de-duplicated. */
export function rememberActionNodeId(nodeId: string, existing: string[]): string[] {
  const trimmed = nodeId.trim()
  if (!trimmed) return existing
  return [trimmed, ...existing.filter((id) => id !== trimmed)].slice(
    0,
    MAX_RECENT_ACTION_NODES,
  )
}

export function createActionId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID()
  }
  return `action_${Date.now()}_${Math.random().toString(36).slice(2)}`
}

export interface PayloadPathCheck {
  valid: boolean
  message: string
}

/**
 * Validates a payload path against the inputs available on the page.
 *
 * An empty path is treated as valid — the field is optional until filled.
 */
export function validatePayloadPath(path: string, inputNames: string[]): PayloadPathCheck {
  const trimmed = path.trim()
  if (!trimmed) return { valid: true, message: '' }

  if (trimmed.startsWith('inputs.')) {
    const segments = trimmed.slice('inputs.'.length).split('.')
    const name = segments[0]?.trim()
    if (!name) return { valid: false, message: 'Select an input name.' }
    if (!inputNames.includes(name)) {
      return { valid: false, message: `Unknown input '${name}'.` }
    }
    if (segments.some((segment) => !segment.trim())) {
      return { valid: false, message: 'Input path segments cannot be empty.' }
    }
    return { valid: true, message: '' }
  }

  if (trimmed.startsWith('context.')) {
    const rest = trimmed.slice('context.'.length)
    if (!rest.trim()) return { valid: false, message: 'Context path is required.' }
    if (rest.split('.').some((segment) => !segment.trim())) {
      return { valid: false, message: 'Context path segments cannot be empty.' }
    }
    return { valid: true, message: '' }
  }

  return { valid: false, message: 'Path must start with inputs. or context.' }
}

export interface PayloadEntry {
  key: string
  path: string
}

/** Ensures every action carries a stable id, so React keys and patches line up. */
export function withActionIds(actions: InspectorAction[]): InspectorAction[] {
  return actions.map((action) => ({
    ...action,
    action_id: action.action_id ?? createActionId(),
  }))
}

/** Falls back to the index so an id-less action is still addressable. */
export function actionKey(action: InspectorAction, index: number): string {
  return action.action_id ?? `action-${index}`
}

export function patchAction(
  actions: InspectorAction[],
  actionId: string,
  patch: Partial<InspectorAction>,
): InspectorAction[] {
  return actions.map((action, index) =>
    actionKey(action, index) === actionId ? { ...action, ...patch } : action,
  )
}

export function patchActionSignal(
  actions: InspectorAction[],
  actionId: string,
  patch: Partial<NonNullable<InspectorAction['signal']>>,
): InspectorAction[] {
  return actions.map((action, index) => {
    if (actionKey(action, index) !== actionId) return action
    return { ...action, signal: { ...(action.signal ?? {}), ...patch } }
  })
}

/** Reads an action's payload map as ordered editable entries. */
export function payloadEntriesOf(action: InspectorAction): PayloadEntry[] {
  const map = action.signal?.payload_map
  if (!map || typeof map !== 'object') return []
  return Object.entries(map as Record<string, unknown>).map(([key, value]) => ({
    key,
    path: String(value ?? ''),
  }))
}

/**
 * Writes editable entries back onto an action.
 *
 * Entries with a blank key are dropped, and an empty map removes `payload_map`
 * entirely rather than persisting `{}`.
 */
export function setPayloadMap(
  actions: InspectorAction[],
  actionId: string,
  entries: PayloadEntry[],
): InspectorAction[] {
  const payloadMap: Record<string, string> = {}
  entries.forEach((entry) => {
    const key = entry.key.trim()
    if (key) payloadMap[key] = entry.path.trim()
  })

  return actions.map((action, index) => {
    if (actionKey(action, index) !== actionId) return action
    const signal = { ...(action.signal ?? {}) }
    if (Object.keys(payloadMap).length === 0) {
      delete signal.payload_map
    } else {
      signal.payload_map = payloadMap
    }
    return { ...action, signal }
  })
}

export function appendAction(actions: InspectorAction[]): InspectorAction[] {
  return [
    ...actions,
    { action_id: createActionId(), trigger: 'on_click', signal: { type: 'submit_node' } },
  ]
}

export function removeActionById(
  actions: InspectorAction[],
  actionId: string,
): InspectorAction[] {
  return actions.filter((action, index) => actionKey(action, index) !== actionId)
}
