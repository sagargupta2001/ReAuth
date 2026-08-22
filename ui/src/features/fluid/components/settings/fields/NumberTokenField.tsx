import { Input } from '@/components/input'
import { FieldLabel } from '@/features/fluid/components/settings/FieldLabel'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'
import { readTokenString } from '@/features/fluid/lib/tokenAccess'
import type { NumberSettingsField } from '@/features/fluid/model/settingsFields'

export function NumberTokenField({ field }: { field: NumberSettingsField }) {
  const { tokens, setToken } = useThemeSettings()

  return (
    <div className="space-y-2">
      {field.label && (
        <FieldLabel htmlFor={field.id} label={field.label} icon={field.icon} hint={field.hint} />
      )}
      <Input
        id={field.id}
        type="number"
        min={field.min}
        max={field.max}
        step={field.step}
        placeholder={field.placeholder}
        value={readTokenString(tokens, field.group, field.token)}
        onChange={(event) => {
          const raw = event.target.value
          setToken(field.group, field.token, raw === '' ? '' : Number(raw))
        }}
      />
    </div>
  )
}
