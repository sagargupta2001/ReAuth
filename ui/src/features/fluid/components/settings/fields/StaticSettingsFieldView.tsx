import { Input } from '@/components/input'
import { FieldLabel } from '@/features/fluid/components/settings/FieldLabel'
import type { StaticSettingsField } from '@/features/fluid/model/settingsFields'

/** Read-only placeholder for a token the builder does not expose yet. */
export function StaticSettingsFieldView({ field }: { field: StaticSettingsField }) {
  return (
    <div className="space-y-2">
      {field.label && (
        <FieldLabel htmlFor={field.id} label={field.label} icon={field.icon} hint={field.hint} />
      )}
      <Input id={field.id} value={field.value} readOnly disabled />
    </div>
  )
}
