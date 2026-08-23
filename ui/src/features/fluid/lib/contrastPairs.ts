import type { ColorRef, ColorContrastPair } from '@/features/fluid/model/settingsFields'
import { readTokenString } from '@/features/fluid/lib/tokenAccess'
import { contrastRatio } from '@/lib/colorUtils'

export interface ContrastResult {
  label: string
  minRatio: number
  /** Null when either colour could not be resolved to something measurable. */
  ratio: number | null
  passes: boolean
}

export function resolveColorRef(tokens: Record<string, unknown>, ref: ColorRef): string {
  if ('literal' in ref) return ref.literal
  return readTokenString(tokens, ref.group, ref.token) || ref.fallback
}

export function evaluateContrastPair(
  tokens: Record<string, unknown>,
  pair: ColorContrastPair,
): ContrastResult {
  const ratio = contrastRatio(
    resolveColorRef(tokens, pair.foreground),
    resolveColorRef(tokens, pair.background),
  )
  return {
    label: pair.label,
    minRatio: pair.minRatio,
    ratio,
    // An unmeasurable pair is not reported as a failure — we cannot say either way.
    passes: ratio === null ? true : ratio >= pair.minRatio,
  }
}

export function evaluateContrastPairs(
  tokens: Record<string, unknown>,
  pairs: readonly ColorContrastPair[],
): ContrastResult[] {
  return pairs.map((pair) => evaluateContrastPair(tokens, pair))
}
