import { TokenSettingsForm } from '@/features/realm/forms/TokenSettingsForm.tsx'
import { useHashScrollHighlight } from '@/shared/hooks/useHashScrollHighlight'

export function TokenSettingsPage() {
  useHashScrollHighlight()

  return (
    <div className="max-w-4xl p-12">
      <TokenSettingsForm />
    </div>
  )
}
