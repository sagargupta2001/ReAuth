import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/select'
import { FieldLabel } from '@/features/fluid/components/settings/FieldLabel'
import { useThemeSettings } from '@/features/fluid/components/settings/themeSettingsContext'
import { readTokenString } from '@/features/fluid/lib/tokenAccess'
import type { SelectSettingsField } from '@/features/fluid/model/settingsFields'

export function SelectTokenField({ field }: { field: SelectSettingsField }) {
  const { tokens, setToken } = useThemeSettings()
  const value = readTokenString(tokens, field.group, field.token, field.fallback)

  return (
    <div className="space-y-2">
      {field.label && (
        <FieldLabel htmlFor={field.id} label={field.label} icon={field.icon} hint={field.hint} />
      )}
      <Select
        value={value}
        onValueChange={(next) => setToken(field.group, field.token, next)}
      >
        <SelectTrigger id={field.id} className="bg-background h-8 text-xs">
          <SelectValue placeholder={field.placeholder} />
        </SelectTrigger>
        <SelectContent>
          {field.options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
