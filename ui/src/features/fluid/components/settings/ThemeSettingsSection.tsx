import { ThemeSettingsField } from '@/features/fluid/components/settings/ThemeSettingsField'
import type { SettingsSection } from '@/features/fluid/model/settingsFields'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'

/** One card in the theme settings panel, rendered from the schema. */
export function ThemeSettingsSection({ section }: { section: SettingsSection }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-sm">{section.title}</CardTitle>
        <CardDescription>{section.description}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {section.fields.map((field) => (
          <ThemeSettingsField key={field.id} field={field} />
        ))}
      </CardContent>
    </Card>
  )
}
