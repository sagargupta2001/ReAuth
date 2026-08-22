import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { SectionCard } from '@/shared/ui/section-card'

/** WCAG AA minimum for normal-sized body text. */
const AA_NORMAL_TEXT = 4.5

interface ContrastCardProps {
  /** Measured ratio, or null when either colour could not be resolved. */
  ratio: number | null
}

/** Contrast report for the selected Text node against the page background. */
export function ContrastCard({ ratio }: ContrastCardProps) {
  return (
    <SectionCard
      title="Accessibility"
      description="Basic contrast check for text color."
      contentClassName="space-y-3 text-xs"
    >
      <div className="flex items-center justify-between">
        <span className="text-muted-foreground">Contrast ratio</span>
        <span className="font-semibold">
          {ratio === null ? 'Unavailable' : `${ratio.toFixed(2)}:1`}
        </span>
      </div>
      <p className="text-muted-foreground">
        AA guidance for normal text is {AA_NORMAL_TEXT}:1 or higher.
      </p>
      {ratio !== null && ratio < AA_NORMAL_TEXT && (
        <Alert variant="destructive">
          <AlertTitle>Low contrast</AlertTitle>
          <AlertDescription>
            Increase text color contrast or adjust the background color.
          </AlertDescription>
        </Alert>
      )}
    </SectionCard>
  )
}
