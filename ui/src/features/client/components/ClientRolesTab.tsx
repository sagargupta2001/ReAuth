import { RolesTable } from '@/features/roles/components/RolesTable.tsx'

interface ClientRolesTabProps {
  clientId: string
}

export function ClientRolesTab({ clientId }: ClientRolesTabProps) {
  return (
    <div className="flex h-full w-full flex-col gap-4 p-6">
      <RolesTable clientId={clientId} />
    </div>
  )
}
