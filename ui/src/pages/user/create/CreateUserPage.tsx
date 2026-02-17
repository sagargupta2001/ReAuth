import { ArrowLeft } from 'lucide-react'

import { buttonVariants } from '@/components/button'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { CreateUserForm } from '@/features/user/forms/CreateUserForm.tsx'
import { cn } from '@/lib/utils'

export function CreateUserPage() {
  return (
    <div className="w-full p-12">
      <div className="mb-2">
        <RealmLink
          to="/users"
          className={cn(
            buttonVariants({ variant: 'link', size: 'sm' }),
            'text-muted-foreground hover:text-foreground gap-2 pl-0',
          )}
        >
          <ArrowLeft className="h-4 w-4" /> Back to Users
        </RealmLink>
      </div>
      <CreateUserForm />
    </div>
  )
}
