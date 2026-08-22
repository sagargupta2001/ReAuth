/**
 * Admin design tokens a theme colour can point at instead of a literal value.
 *
 * Seeded themes use these (`var(--primary)` and friends) so a fresh theme
 * inherits the product palette. A theme that references a token follows the
 * product's palette; a theme with a literal hex pins its own colour.
 */
export interface DesignColorToken {
  /** The CSS value stored in the theme draft. */
  value: string
  label: string
}

export const DESIGN_COLOR_TOKENS: readonly DesignColorToken[] = [
  { value: 'var(--primary)', label: 'Primary' },
  { value: 'var(--background)', label: 'Background' },
  { value: 'var(--foreground)', label: 'Foreground' },
  { value: 'var(--card)', label: 'Card / Surface' },
  { value: 'var(--muted)', label: 'Muted' },
  { value: 'var(--muted-foreground)', label: 'Muted Foreground' },
  { value: 'var(--accent)', label: 'Accent' },
  { value: 'var(--destructive)', label: 'Destructive' },
  { value: 'var(--border)', label: 'Border' },
]

/** True when a stored colour references a design token rather than a literal. */
export function isDesignTokenValue(value: string): boolean {
  return value.trim().startsWith('var(')
}

export function findDesignToken(value: string): DesignColorToken | undefined {
  const normalized = value.trim()
  return DESIGN_COLOR_TOKENS.find((token) => token.value === normalized)
}
