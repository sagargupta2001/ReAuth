import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { CreateRoleDialog } from '@/features/roles/components/CreateRoleDialog'
import { RolesTable } from '@/features/roles/components/RolesTable.tsx'

interface ClientRolesTabProps {
  clientId: string
}

export function ClientRolesTab({ clientId }: ClientRolesTabProps) {
  return (
    <div className="p-6">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-6">
          <div className="space-y-1">
            <CardTitle>Client Roles</CardTitle>
            <CardDescription>
              Manage roles specific to this application (e.g., "manager", "viewer").
            </CardDescription>
          </div>

          <CreateRoleDialog clientId={clientId} />
        </CardHeader>

        <CardContent>
          <RolesTable clientId={clientId} />
        </CardContent>
      </Card>
    </div>
  )
}
