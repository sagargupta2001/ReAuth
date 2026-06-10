import { EmailSection } from './profile/EmailSection.tsx'
import { ProfileSection } from './profile/ProfileSection.tsx'

export function UseProfileTab({ userId }: { userId: string }) {
  return (
    <div className="flex max-w-2xl flex-col gap-6">
      <ProfileSection userId={userId} />
      <EmailSection userId={userId} />
    </div>
  )
}
