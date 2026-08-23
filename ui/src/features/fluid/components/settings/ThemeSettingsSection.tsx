import { SectionCard } from '@/shared/ui/section-card'
import { ThemeSettingsField } from '@/features/fluid/components/settings/ThemeSettingsField'
import type { SettingsSection } from '@/features/fluid/model/settingsFields'

/** One card in the theme settings panel, rendered from the schema. */
export function ThemeSettingsSection({ section }: { section: SettingsSection }) {
  return (
    <SectionCard title={section.title} description={section.description}>
      {section.fields.map((field) => (
        <ThemeSettingsField key={field.id} field={field} />
      ))}
    </SectionCard>
  )
}
