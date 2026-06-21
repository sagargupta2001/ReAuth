import { useEffect, useMemo, useState } from 'react'

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
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Group } from '@/entities/group/model/types'
import { useDeleteGroup } from '@/features/group/api/useDeleteGroup'
import { useGroupDeleteSummary } from '@/features/group/api/useGroupDeleteSummary'
import { useUpdateGroup } from '@/features/group/api/useUpdateGroup'
import { useGroupTreeStore } from '@/features/group-tree/model/groupTreeStore'
import { findNode, removeNode } from '@/features/group-tree/lib/tree-utils'
import type { GroupTreeNode } from '@/features/group-tree/model/types'
import { type GroupFormValues, groupSchema } from '@/features/group/schema/create.schema'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { Checkbox } from '@/shared/ui/checkbox'
import { FormInput } from '@/shared/ui/form-input'
import { FormTextarea } from '@/shared/ui/form-textarea'
import { Separator } from '@/shared/ui/separator'

interface GroupSettingsTabProps {
  group: Group
}

const EMPTY_IDS: string[] = []

export function GroupSettingsTab({ group }: GroupSettingsTabProps) {
  const navigate = useRealmNavigate()
  const realm = useActiveRealm()
  const mutation = useUpdateGroup(group.id)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [cascade, setCascade] = useState(false)
  const updateTreeCache = useGroupTreeStore((state) => state.updateTree)
  const expandedByRealm = useGroupTreeStore((state) => state.expandedByRealm)
  const expandedIdsList = expandedByRealm[realm] ?? EMPTY_IDS
  const setExpanded = useGroupTreeStore((state) => state.setExpanded)
  const deleteGroup = useDeleteGroup(group.id)
  const { data: summary, isLoading: summaryLoading } = useGroupDeleteSummary(group.id, deleteOpen)

  const hasDescendants = (summary?.descendant_count || 0) > 0
  const deleteDisabled = summaryLoading || (hasDescendants && !cascade)

  const impactLabel = useMemo(() => {
    if (!summary) return ''
    const totalGroups = summary.descendant_count + 1
    return `${totalGroups} group${totalGroups === 1 ? '' : 's'}`
  }, [summary])

  const form = useForm<GroupFormValues>({
    resolver: zodResolver(groupSchema),
    defaultValues: {
      name: group.name,
      description: group.description || '',
    },
  })

  useEffect(() => {
    form.reset({
      name: group.name,
      description: group.description || '',
    })
  }, [group, form])

  const onSubmit = (values: GroupFormValues) => {
    mutation.mutate(values, {
      onSuccess: () => {
        form.reset(values)
      },
    })
  }

  useFormPersistence(form, onSubmit, mutation.isPending)

  const handleConfirmDelete = () => {
    deleteGroup.mutate(
      { cascade: hasDescendants ? cascade : false },
      {
        onSuccess: () => {
          let nextTree: GroupTreeNode[] = []
          updateTreeCache(realm, (prev) => {
            const result = removeNode(prev, group.id)
            nextTree = result.tree
            return result.tree
          })
          if (nextTree.length > 0) {
            const remaining = expandedIdsList.filter((id) => findNode(nextTree, id))
            setExpanded(realm, remaining)
          } else {
            setExpanded(realm, [])
          }
          setDeleteOpen(false)
          setCascade(false)
          navigate('/groups')
        },
      },
    )
  }

  return (
    <div className="max-w-4xl space-y-6 p-6">
      <Card>
        <CardHeader>
          <CardTitle>Basic Information</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="bg-primary-foreground rounded-2xl p-4">
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
                <FormInput
                  control={form.control}
                  name="name"
                  label="Group Name"
                  placeholder="e.g. product-team"
                  description="The display name shown in the admin UI."
                />

                <FormTextarea
                  control={form.control}
                  name="description"
                  label="Description"
                  placeholder="Describe who should belong to this group..."
                  className="min-h-[120px]"
                />
              </form>
            </Form>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Danger Zone</CardTitle>
          <CardDescription>
            Delete this group and unassign its members and roles.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="border-destructive/30 bg-destructive/5 flex flex-wrap items-center justify-between gap-4 rounded-2xl border p-4">
            <div>
              <p className="text-sm font-medium">Delete group</p>
              <p className="text-muted-foreground text-sm">
                Permanently removes the group and unassigns any roles or members linked to it.
              </p>
            </div>
            <Button type="button" variant="destructive" onClick={() => setDeleteOpen(true)}>
              <Trash2 className="h-4 w-4" />
              Delete Group
            </Button>
          </div>
        </CardContent>
      </Card>

      <Dialog
        open={deleteOpen}
        onOpenChange={(open) => {
          setDeleteOpen(open)
          if (!open) setCascade(false)
        }}
      >
        <DialogContent className="sm:max-w-[520px]">
          <DialogHeader className="pt-6 pl-6">
            <DialogTitle>Delete group</DialogTitle>
            <DialogDescription>
              This permanently removes the group and unassigns any roles or members linked to it.
            </DialogDescription>
          </DialogHeader>

          <Separator className="my-1" />

          <div className="px-6 pb-6">
            {summaryLoading ? (
              <div className="text-muted-foreground text-sm">Loading impact...</div>
            ) : summary ? (
              <div className="space-y-3 text-sm">
                <div className="grid grid-cols-2 gap-2">
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Groups affected</div>
                    <div className="font-medium">{impactLabel}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Sub-groups</div>
                    <div className="font-medium">{summary.descendant_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Members affected</div>
                    <div className="font-medium">{summary.member_count}</div>
                  </div>
                  <div className="rounded-md border px-3 py-2">
                    <div className="text-muted-foreground text-xs">Roles affected</div>
                    <div className="font-medium">{summary.role_count}</div>
                  </div>
                </div>

                {hasDescendants ? (
                  <label className="flex items-start gap-3 rounded-md border p-3">
                    <Checkbox
                      checked={cascade}
                      onCheckedChange={(value) => setCascade(Boolean(value))}
                    />
                    <div className="space-y-1">
                      <div className="font-medium">
                        Also delete {summary.descendant_count} sub-group
                        {summary.descendant_count === 1 ? '' : 's'}
                      </div>
                      <div className="text-muted-foreground text-xs">
                        Required to remove nested groups inside this hierarchy.
                      </div>
                    </div>
                  </label>
                ) : null}
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
              disabled={deleteDisabled || deleteGroup.isPending || !summary}
            >
              {deleteGroup.isPending ? 'Deleting...' : 'Delete Group'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
