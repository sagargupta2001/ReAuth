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

export function resolveThemeColor(
  value: string,
  mode: string,
  fallback: string,
  legacy: string[],
) {
  const trimmed = value.trim()
  if (!trimmed) return fallback
  const hslVarMatch = trimmed.match(/^hsl\(\s*(var\(--[^)]+\))\s*\)$/i)
  if (hslVarMatch) {
    return hslVarMatch[1]
  }
  const normalized = trimmed.toLowerCase()
  if (mode === 'dark' && legacy.includes(normalized)) {
    return fallback
  }
  return trimmed
}

export function resolveThemeMode(mode: string) {
  if (mode !== 'auto') return mode
  if (typeof window === 'undefined') return 'light'
  if (document?.documentElement?.classList?.contains('dark')) return 'dark'
  if (document?.documentElement?.classList?.contains('light')) return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function resolveInputType(props: Record<string, unknown>, name: string) {
  const explicit = String(props.input_type || '').trim()
  if (explicit) return explicit
  if (name.toLowerCase().includes('password')) return 'password'
  return 'text'
}
