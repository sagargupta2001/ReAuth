import { IdentityProviderForm } from '@/features/identity-provider/forms/IdentityProviderForm'
import { Main } from '@/widgets/Layout/Main'

export function CreateIdentityProviderPage() {
  return (
    <Main className="flex flex-1 flex-col gap-6 p-12">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Create Identity Provider</h2>
        <p className="text-muted-foreground">
          Add a new inbound OAuth or OIDC provider for browser sign-in and account linking flows.
        </p>
      </div>
      <IdentityProviderForm />
    </Main>
  )
}
