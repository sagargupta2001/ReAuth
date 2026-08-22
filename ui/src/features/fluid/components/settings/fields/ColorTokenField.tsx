import { ColorPicker } from '@/features/fluid/components/controls/ColorPicker'
import { FieldLabel } from '@/features/fluid/components/settings/FieldLabel'
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
      <ColorPicker
        id={field.id}
        ariaLabel={field.label ? `${field.label} color` : undefined}
        value={value}
        fallback={field.fallback}
        onChange={(next) => setToken(field.group, field.token, next)}
      />
    </div>
  )
}
