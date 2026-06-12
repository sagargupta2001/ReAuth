import { CreateRoleDialog } from '@/features/roles/components/CreateRoleDialog'

interface RolesPrimaryButtonsProps {
  clientId?: string
}

export function RolesPrimaryButtons({ clientId }: RolesPrimaryButtonsProps) {
  return (
    <div className="flex gap-2">
      <CreateRoleDialog clientId={clientId} />
    </div>
  )
}
