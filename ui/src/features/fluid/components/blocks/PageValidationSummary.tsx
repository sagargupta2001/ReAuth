import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'

interface PageValidationSummaryProps {
  errors: ThemeValidationError[]
}

/** Banner for validation errors that are not attached to a single node. */
export function PageValidationSummary({ errors }: PageValidationSummaryProps) {
  if (errors.length === 0) return null

  return (
    <div
      role="status"
      className="border-destructive/40 bg-destructive/10 text-destructive mb-3 rounded-md border px-2 py-1 text-[11px]"
    >
      {errors.length} page-level validation issue(s). Check the inspector.
    </div>
  )
}
