import { SecurityHeadersSettingsForm } from '@/features/realm/forms/SecurityHeadersSettingsForm.tsx'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function SecurityHeadersSettingsPage() {
  useHashScrollHighlight()

  return (
    <div className="max-w-4xl p-12">
      <SecurityHeadersSettingsForm />
    </div>
  )
}
