import { useEffect, useState } from 'react'

import { Trash2 } from 'lucide-react'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Form } from '@/components/form'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useDeleteRole } from '@/features/roles/api/useDeleteRole'
import { useRoleDeleteSummary } from '@/features/roles/api/useRoleDeleteSummary'
import type { Role } from '@/features/roles/api/useRoles.ts'
import { useUpdateRole } from '@/features/roles/api/useUpdateRole'
import { type RoleFormValues, roleSchema } from '@/features/roles/schema/create.schema'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'

interface RoleSettingsTabProps {
  role: Role
  /** Set when the role was opened from a client, so delete returns to that
   * client's Roles tab instead of the realm roles list. */
  clientId?: string
}

export function RoleSettingsTab({ role, clientId }: RoleSettingsTabProps) {
  const navigate = useRealmNavigate()
  const mutation = useUpdateRole(role.id)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const deleteRole = useDeleteRole(role.id)
  const { data: summary, isLoading: summaryLoading } = useRoleDeleteSummary(role.id, deleteOpen)

  const form = useForm<RoleFormValues>({
    resolver: zodResolver(roleSchema),
    defaultValues: {
      name: role.name,
      description: role.description || '',
    },
  })

  useEffect(() => {
    form.reset({
      name: role.name,
      description: role.description || '',
    })
  }, [role, form])

  const onSubmit = (values: RoleFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => {
        form.reset(values)
      },
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  const handleConfirmDelete = () => {
    deleteRole.mutate(undefined, {
      onSuccess: () => {
        setDeleteOpen(false)
        navigate(clientId ? `/clients/${clientId}/roles` : '/roles')
      },
    })
  }

  return (
    <div className="max-w-4xl space-y-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Basic Information</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="bg-primary-foreground p-4 rounded-2xl space-y-4">
                <FormInput
                  control={form.control}
                  name="name"
                  label="Role Name"
                  placeholder="e.g. content_manager"
                  description="The unique identifier used in code checks. Changing this may break existing authorization logic."
                />

                <FormTextarea
                  control={form.control}
                  name="description"
                  label="Description"
                  placeholder="Describe the purpose and responsibilities of this role..."
                  className="min-h-[120px]"
                />
              </div>
            </CardContent>
          </Card>
        </form>
      </Form>

      <Card>
        <CardHeader>
          <CardTitle>Danger Zone</CardTitle>
          <CardDescription>
            Delete this role and remove its assignments, composites, and permissions.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="border-destructive/30 bg-destructive/5 flex flex-wrap items-center justify-between gap-4 rounded-2xl border p-4">
            <div>
              <p className="text-sm font-medium">Delete role</p>
              <p className="text-muted-foreground text-sm">
                Permanently removes the role and clears all user assignments and permissions linked to it.
              </p>
            </div>
            <Button type="button" variant="destructive" onClick={() => setDeleteOpen(true)}>
              <Trash2 className="h-4 w-4" />
              Delete Role
            </Button>
          </div>
        </CardContent>
      </Card>

      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent className="sm:max-w-[520px]">
          <DialogHeader className="px-6 pt-6">
            <DialogTitle>Delete role</DialogTitle>
            <DialogDescription>
              This permanently removes the role and clears assignments, composites, and permissions
              linked to it.
            </DialogDescription>
          </DialogHeader>

          <div className="px-6 pb-2">
            {summaryLoading ? (
              <div className="text-muted-foreground text-sm">Loading impact...</div>
            ) : summary ? (
              <div className="space-y-3 text-sm">
                <div className="grid grid-cols-2 gap-2">
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Direct users</div>
                    <div className="font-medium">{summary.direct_user_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Effective users</div>
                    <div className="font-medium">{summary.effective_user_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Groups assigned</div>
                    <div className="font-medium">{summary.group_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Parent composites</div>
                    <div className="font-medium">{summary.parent_role_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Child composites</div>
                    <div className="font-medium">{summary.child_role_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Permissions</div>
                    <div className="font-medium">{summary.permission_count}</div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-destructive text-sm">Unable to load delete impact.</div>
            )}
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmDelete}
              disabled={summaryLoading || deleteRole.isPending || !summary}
            >
              {deleteRole.isPending ? 'Deleting...' : 'Delete Role'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
