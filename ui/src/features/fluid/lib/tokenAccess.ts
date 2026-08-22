import {
  DEFAULT_LAYOUT_SLOTS,
  LayoutKey,
  type TokenGroup,
} from '@/features/fluid/model/tokens'
import { DEFAULT_LAYOUT_SHELL } from '@/features/fluid/model/layoutShells'

type UnknownRecord = Record<string, unknown>

/** Reads a nested token group, treating anything that is not a plain object as absent. */
export function readTokenGroup(tokens: UnknownRecord, group: TokenGroup): UnknownRecord {
  const value = tokens[group]
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as UnknownRecord
  }
  return {}
}

/** Reads a single token as a string, falling back when it is missing or not a string. */
export function readTokenString(
  tokens: UnknownRecord,
  group: TokenGroup,
  token: string,
  fallback = '',
): string {
  const value = readTokenGroup(tokens, group)[token]
  if (typeof value === 'string') return value
  if (typeof value === 'number') return String(value)
  return fallback
}

/** Returns a new token record with a single token replaced. */
export function withTokenValue(
  tokens: UnknownRecord,
  group: TokenGroup,
  token: string,
  value: unknown,
): UnknownRecord {
  return {
    ...tokens,
    [group]: {
      ...readTokenGroup(tokens, group),
      [token]: value,
    },
  }
}

export function readLayoutShell(layout: UnknownRecord): string {
  const shell = layout[LayoutKey.Shell]
  return typeof shell === 'string' ? shell : DEFAULT_LAYOUT_SHELL
}

/** Returns a new layout record with the shell replaced and slots normalized. */
export function withLayoutShell(layout: UnknownRecord, shell: string): UnknownRecord {
  const slots = layout[LayoutKey.Slots]
  return {
    ...layout,
    [LayoutKey.Shell]: shell,
    [LayoutKey.Slots]: Array.isArray(slots) ? slots : [...DEFAULT_LAYOUT_SLOTS],
  }
}
