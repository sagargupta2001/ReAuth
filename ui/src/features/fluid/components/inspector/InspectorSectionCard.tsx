import { InspectorField } from '@/features/fluid/components/inspector/InspectorField'
import type { InspectorSection } from '@/features/fluid/model/inspectorFields'
import { SectionCard } from '@/shared/ui/section-card'

/** One card in the inspector's Properties tab, rendered from the schema. */
export function InspectorSectionCard({ section }: { section: InspectorSection }) {
  return (
    <SectionCard title={section.title} description={section.description}>
      {section.fields.map((field) => (
        <InspectorField key={field.id} field={field} />
      ))}
    </SectionCard>
  )
}
