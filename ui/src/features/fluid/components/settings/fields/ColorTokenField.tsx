import { FieldLabel } from '@/features/fluid/components/controls/FieldLabel'
import { ThemeColorControl } from '@/features/fluid/components/controls/ThemeColorControl'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'
import { readTokenString } from '@/features/fluid/lib/tokenAccess'
import type { ColorSettingsField } from '@/features/fluid/model/settingsFields'

export function ColorTokenField({ field }: { field: ColorSettingsField }) {
  const { tokens, setToken } = useThemeSettings()
  const value = readTokenString(tokens, field.group, field.token)

  return (
    <div className="space-y-2">
      {field.label && (
        <FieldLabel htmlFor={field.id} label={field.label} icon={field.icon} hint={field.hint} />
      )}
      <ThemeColorControl
        id={field.id}
        value={value}
        fallback={field.fallback}
        ariaLabel={field.label}
        onChange={(next) => setToken(field.group, field.token, next)}
      />
    </div>
  )
}
