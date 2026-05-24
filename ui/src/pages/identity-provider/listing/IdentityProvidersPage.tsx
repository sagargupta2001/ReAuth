import { IdentityProvidersList } from '@/features/identity-provider/components/IdentityProvidersList'
import { IdentityProvidersPrimaryButtons } from '@/features/identity-provider/components/IdentityProvidersPrimaryButtons'
import { Main } from '@/widgets/Layout/Main'

export function IdentityProvidersPage() {
  return (
    <Main className="flex flex-1 flex-col gap-4 p-12 sm:gap-6">
      <div className="flex flex-wrap items-end justify-between gap-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Identity Providers</h2>
          <p className="text-muted-foreground">
            Manage inbound OAuth and OIDC providers used for social, workforce, and federation login.
          </p>
        </div>
        <IdentityProvidersPrimaryButtons />
      </div>
      <IdentityProvidersList />
    </Main>
  )
}
