import { EmailSection } from './profile/EmailSection.tsx'
import { PhoneNumberSection } from './profile/PhoneNumberSection.tsx'
import { ProfileSection } from './profile/ProfileSection.tsx'

export function UseProfileTab({ userId }: { userId: string }) {
  return (
    <div className="flex flex-col gap-6">
      <ProfileSection userId={userId} />
      <EmailSection userId={userId} />
      <PhoneNumberSection userId={userId} />
    </div>
  )
}
