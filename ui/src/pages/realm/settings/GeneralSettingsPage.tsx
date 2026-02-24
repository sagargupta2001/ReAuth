import { GeneralSettingsForm } from '@/features/realm/forms/GeneralSettingsForm.tsx'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function GeneralSettingsPage() {
  useHashScrollHighlight()

  return (
    <div className="max-w-4xl p-12">
      <GeneralSettingsForm />
    </div>
  )
}
