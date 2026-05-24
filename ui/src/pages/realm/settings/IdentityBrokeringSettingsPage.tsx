import { IdpSettingsForm } from '@/features/realm/forms/IdpSettingsForm.tsx'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function IdentityBrokeringSettingsPage() {
  useHashScrollHighlight()

  return (
    <div className="max-w-4xl p-12">
      <IdpSettingsForm />
    </div>
  )
}
