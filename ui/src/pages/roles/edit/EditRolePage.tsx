import { ArrowLeft } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { buttonVariants } from '@/components/button'
import { RealmLink } from '@/entities/realm/lib/navigation'
import { cn } from '@/lib/utils'
import { EditRoleForm } from '@/features/roles/forms/EditRoleForm.tsx'

export function EditRolePage() {
  const { roleId } = useParams<{ roleId: string }>()

  if (!roleId) return null

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
      <EditRoleForm roleId={roleId} />
    </div>
  )
}
