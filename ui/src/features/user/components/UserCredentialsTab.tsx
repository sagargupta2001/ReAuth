import { useUserCredentials } from '@/features/user/api/useUserCredentials'
import { FederatedIdentitiesSection } from '@/features/user/components/credentials/FederatedIdentitiesSection'
import { PasskeysSection } from '@/features/user/components/credentials/PasskeysSection'
import { PasswordSection } from '@/features/user/components/credentials/PasswordSection'
import { Skeleton } from '@/shared/ui/skeleton'

interface UserCredentialsTabProps {
  userId: string
}

export function UserCredentialsTab({ userId }: UserCredentialsTabProps) {
  const { data, isLoading } = useUserCredentials(userId)

  if (isLoading) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-20" />
        <Skeleton className="h-20" />
        <Skeleton className="h-20" />
      </div>
    )
  }

  return (
    <div className="flex h-full w-full flex-col gap-6">
      <PasswordSection userId={userId} password={data?.password} />
      <PasskeysSection userId={userId} passkeys={data?.passkeys ?? []} />
      <FederatedIdentitiesSection userId={userId} identities={data?.federated_identities ?? []} />
    </div>
  )
}
