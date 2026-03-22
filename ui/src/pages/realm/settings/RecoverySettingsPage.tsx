import { RecoverySettingsForm } from '@/features/realm/forms/RecoverySettingsForm.tsx'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function RecoverySettingsPage() {
  useHashScrollHighlight()

  return (
    <div className="max-w-4xl p-12">
      <RecoverySettingsForm />
    </div>
  )
}
