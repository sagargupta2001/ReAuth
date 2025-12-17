import { ArrowLeft } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { buttonVariants } from '@/components/button'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { EditUserForm } from '@/features/user/edit/EditUserForm'
import { cn } from '@/lib/utils'

export function EditUserPage() {
  const { userId } = useParams<{ userId: string }>()

  if (!userId) return null

  return (
    <div className="w-full py-6">
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
      <EditUserForm userId={userId} />
    </div>
  )
}
