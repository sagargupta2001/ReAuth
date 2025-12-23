import { ArrowLeft } from 'lucide-react'

import { buttonVariants } from '@/components/button'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { cn } from '@/lib/utils'
import { CreateRoleForm } from '@/features/roles/forms/CreateRoleForm.tsx'

export function CreateRolePage() {
  return (
    <div className="w-full py-6">
      <div className="mb-2">
        <RealmLink
          to="/roles"
          className={cn(
            buttonVariants({ variant: 'link', size: 'sm' }),
            'text-muted-foreground hover:text-foreground gap-2 pl-0',
          )}
        >
          <ArrowLeft className="h-4 w-4" /> Back to Roles
        </RealmLink>
      </div>
      <CreateRoleForm />
    </div>
  )
}
