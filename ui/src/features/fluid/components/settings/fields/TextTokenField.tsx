import { Input } from '@/components/input'
import { FieldLabel } from '@/features/fluid/components/controls/FieldLabel'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'
import { readTokenString } from '@/features/fluid/lib/tokenAccess'
import type { TextSettingsField } from '@/features/fluid/model/settingsFields'

export function TextTokenField({ field }: { field: TextSettingsField }) {
  const { tokens, setToken } = useThemeSettings()

  return (
    <div className="space-y-2">
      {field.label && (
        <FieldLabel htmlFor={field.id} label={field.label} icon={field.icon} hint={field.hint} />
      )}
      <Input
        id={field.id}
        value={readTokenString(tokens, field.group, field.token)}
        placeholder={field.placeholder}
        onChange={(event) => setToken(field.group, field.token, event.target.value)}
      />
    </div>
  )
}
