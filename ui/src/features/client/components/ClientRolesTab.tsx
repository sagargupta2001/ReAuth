import { useState } from 'react'

import { Plus } from 'lucide-react'

import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { Dialog, DialogContent, DialogTrigger } from '@/components/dialog'
import { CreateRoleForm } from '@/features/roles/forms/CreateRoleForm.tsx'
import { RolesTable } from '@/features/roles/components/RolesTable.tsx'


interface ClientRolesTabProps {
  clientId: string
}

export function ClientRolesTab({ clientId }: ClientRolesTabProps) {
  const [isCreateOpen, setIsCreateOpen] = useState(false)

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

          <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
            <DialogTrigger asChild>
              <Button className="gap-2">
                <Plus size={16} /> Create Role
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[600px]">
              <CreateRoleForm isDialog={true} clientId={clientId} onSuccess={() => setIsCreateOpen(false)} />
            </DialogContent>
          </Dialog>
        </CardHeader>

        <CardContent>
          <RolesTable clientId={clientId} />
        </CardContent>
      </Card>
    </div>
  )
}