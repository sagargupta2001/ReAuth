import { useUserCredentials } from '@/features/user/api/useUserCredentials'
import { PasswordPolicySection } from '@/features/user/components/settings/PasswordPolicySection'
import { Skeleton } from '@/shared/ui/skeleton'

interface UserSettingsTabProps {
  userId: string
}

export function UserSettingsTab({ userId }: UserSettingsTabProps) {
  const { data, isLoading } = useUserCredentials(userId)

  if (isLoading) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-20" />
        <Skeleton className="h-20" />
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-6">
      <PasswordPolicySection userId={userId} password={data?.password} />
    </div>
  )
}
