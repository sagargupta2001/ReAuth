import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'

export interface ValidationIndex {
  /** Errors that point at a specific node, keyed by node id. */
  byNodeId: Map<string, ThemeValidationError[]>
  /** Errors that belong to the page rather than a single node. */
  pageErrors: ThemeValidationError[]
}

export function buildValidationIndex(errors: ThemeValidationError[]): ValidationIndex {
  const byNodeId = new Map<string, ThemeValidationError[]>()
  const pageErrors: ThemeValidationError[] = []

  errors.forEach((error) => {
    if (!error.nodeId) {
      pageErrors.push(error)
      return
    }
    const existing = byNodeId.get(error.nodeId)
    if (existing) {
      existing.push(error)
      return
    }
    byNodeId.set(error.nodeId, [error])
  })

  return { byNodeId, pageErrors }
}
