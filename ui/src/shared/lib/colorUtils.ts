export function normalizeColorValue(value: string) {
  const hex = value.trim()
  if (/^#([0-9a-f]{3}|[0-9a-f]{6})$/i.test(hex)) {
    return hex
  }
  return '#111827'
}

type Rgb = { r: number; g: number; b: number }

export function parseColor(value: string): Rgb | null {
  const input = value.trim()
  const hexMatch = input.match(/^#([0-9a-f]{3}|[0-9a-f]{6})$/i)
  if (hexMatch) {
    const hex = hexMatch[1]
    const expanded =
      hex.length === 3
        ? hex
            .split('')
            .map((char) => `${char}${char}`)
            .join('')
        : hex
    const r = Number.parseInt(expanded.slice(0, 2), 16)
    const g = Number.parseInt(expanded.slice(2, 4), 16)
    const b = Number.parseInt(expanded.slice(4, 6), 16)
    return { r, g, b }
  }
  const rgbMatch = input.match(
    /^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)(?:\s*,\s*[\d.]+)?\s*\)$/i,
  )
  if (rgbMatch) {
    return {
      r: Number.parseInt(rgbMatch[1], 10),
      g: Number.parseInt(rgbMatch[2], 10),
      b: Number.parseInt(rgbMatch[3], 10),
    }
  }
  return null
}

export function relativeLuminance({ r, g, b }: Rgb) {
  const toLinear = (channel: number) => {
    const normalized = channel / 255
    return normalized <= 0.03928
      ? normalized / 12.92
      : Math.pow((normalized + 0.055) / 1.055, 2.4)
  }
  const rLin = toLinear(r)
  const gLin = toLinear(g)
  const bLin = toLinear(b)
  return 0.2126 * rLin + 0.7152 * gLin + 0.0722 * bLin
}

/**
 * Resolves a CSS colour expression to something `parseColor` understands.
 *
 * `var(--token)` values are looked up on the document root, falling back to the
 * `var()` fallback argument when there is no document (SSR, tests) or the custom
 * property is not defined.
 */
export function resolveCssColor(value: string): string | null {
  const trimmed = value.trim()
  if (!trimmed) return null

  const varMatch = trimmed.match(/^var\(\s*(--[\w-]+)\s*(?:,\s*([^)]+))?\)$/)
  if (!varMatch) return trimmed

  const [, property, fallback] = varMatch
  const declared =
    typeof document === 'undefined'
      ? ''
      : getComputedStyle(document.documentElement).getPropertyValue(property).trim()

  const resolved = declared || fallback?.trim() || ''
  if (!resolved) return null
  // A custom property may point at another one, e.g. --bg-table: var(--table-background).
  return resolved.startsWith('var(') ? resolveCssColor(resolved) : resolved
}

/**
 * Returns `color` at the given alpha, or the original value when it cannot be
 * parsed (e.g. an unresolvable `var()`).
 *
 * Used to derive borders and secondary text from a theme's own text colour, so
 * they follow the theme instead of borrowing the admin palette.
 */
export function withAlpha(color: string, alpha: number): string {
  const resolved = resolveCssColor(color)
  const rgb = resolved ? parseColor(resolved) : null
  if (!rgb) return color
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`
}

/** Picks the higher-contrast of black or white for text drawn on `background`. */
export function readableTextOn(background: string, light = '#ffffff', dark = '#111827'): string {
  const resolved = resolveCssColor(background)
  const rgb = resolved ? parseColor(resolved) : null
  if (!rgb) return light
  return relativeLuminance(rgb) > 0.45 ? dark : light
}

export function contrastRatio(foreground: string, background: string) {
  const resolvedForeground = resolveCssColor(foreground)
  const resolvedBackground = resolveCssColor(background)
  if (!resolvedForeground || !resolvedBackground) return null
  const fg = parseColor(resolvedForeground)
  const bg = parseColor(resolvedBackground)
  if (!fg || !bg) return null
  const l1 = relativeLuminance(fg)
  const l2 = relativeLuminance(bg)
  const lighter = Math.max(l1, l2)
  const darker = Math.min(l1, l2)
  return (lighter + 0.05) / (darker + 0.05)
}
