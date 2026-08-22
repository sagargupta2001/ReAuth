import { useMemo } from 'react'

import { AlertTriangle, Check } from 'lucide-react'

import { FieldLabel } from '@/features/fluid/components/controls/FieldLabel'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'
import { evaluateContrastPairs, type ContrastResult } from '@/features/fluid/lib/contrastPairs'
import type { ColorContrastSettingsField } from '@/features/fluid/model/settingsFields'
import { cn } from '@/lib/utils'

/** WCAG contrast report for the colour pairs the renderers actually produce. */
export function ColorContrastField({ field }: { field: ColorContrastSettingsField }) {
  const { tokens } = useThemeSettings()
  const results = useMemo(() => evaluateContrastPairs(tokens, field.pairs), [tokens, field.pairs])
  const failures = results.filter((result) => !result.passes)

  return (
    <div className="space-y-2">
      {field.label && <FieldLabel label={field.label} icon={field.icon} hint={field.hint} />}
      <div className="space-y-1">
        {results.map((result) => (
          <ContrastRow key={result.label} result={result} />
        ))}
      </div>
      {failures.length > 0 && (
        <p className="text-destructive text-[11px]">
          {failures.length} pair(s) below the WCAG AA minimum. Adjust the colours above.
        </p>
      )}
    </div>
  )
}

function ContrastRow({ result }: { result: ContrastResult }) {
  const { label, ratio, minRatio, passes } = result

  return (
    <div className="flex items-center justify-between gap-2 text-[11px]">
      <span className="text-muted-foreground min-w-0 truncate">{label}</span>
      <span
        className={cn(
          'flex shrink-0 items-center gap-1 font-semibold',
          ratio === null ? 'text-muted-foreground' : passes ? 'text-foreground' : 'text-destructive',
        )}
        title={ratio === null ? 'Colour could not be resolved' : `Minimum ${minRatio}:1`}
      >
        {ratio === null ? (
          'n/a'
        ) : (
          <>
            {passes ? (
              <Check className="h-3 w-3" />
            ) : (
              <AlertTriangle className="h-3 w-3" />
            )}
            {ratio.toFixed(2)}:1
          </>
        )}
      </span>
    </div>
  )
}
