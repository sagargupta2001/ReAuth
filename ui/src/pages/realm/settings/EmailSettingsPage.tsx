import { EmailSettingsForm } from '@/features/realm/forms/EmailSettingsForm.tsx'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function EmailSettingsPage() {
  useHashScrollHighlight()

  return (
    <div className="max-w-4xl p-12">
      <EmailSettingsForm />
    </div>
  )
}
