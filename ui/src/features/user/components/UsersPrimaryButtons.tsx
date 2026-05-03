import { CreateUserDialog } from '@/features/user/components/CreateUserDialog'

export function UsersPrimaryButtons() {
  return (
    <div className="flex gap-2">
      <CreateUserDialog />
    </div>
  )
}
