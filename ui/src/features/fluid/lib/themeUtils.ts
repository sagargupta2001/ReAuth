export function getNestedRecord(
  source: Record<string, unknown>,
  key: string,
): Record<string, unknown> {
  const value = source[key]
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as Record<string, unknown>
  }
  return {}
}

/**
 * Resolves a theme colour token to a CSS colour value.
 *
 * Themes have a single appearance, so a literal value is always honoured
 * verbatim — there is no light/dark substitution.
 */
export function resolveThemeColor(value: string, fallback: string) {
  const trimmed = value.trim()
  if (!trimmed) return fallback
  // Legacy themes stored design tokens as hsl(var(--x)); unwrap to var(--x).
  const hslVarMatch = trimmed.match(/^hsl\(\s*(var\(--[^)]+\))\s*\)$/i)
  if (hslVarMatch) {
    return hslVarMatch[1]
  }
  return trimmed
}

export function resolveInputType(props: Record<string, unknown>, name: string) {
  const explicit = String(props.input_type || '').trim()
  if (explicit) return explicit
  if (name.toLowerCase().includes('password')) return 'password'
  return 'text'
}
