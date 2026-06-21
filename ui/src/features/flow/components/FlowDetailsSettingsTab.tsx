import { useEffect, useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { Copy, Trash2 } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import type { FlowDraft } from '@/entities/flow/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useDeleteFlow } from '@/features/flow/api/useDeleteFlow.ts'
import { useUpdateFlow } from '@/features/flow/api/useUpdateFlow.ts'
import { CloneFlowDialog } from '@/features/flow/components/CloneFlowDialog.tsx'
import { FlowSummaryPanel } from '@/features/flow/components/FlowSummaryPanel.tsx'
import {
  type FlowSettingsSchema,
  flowSettingsSchema,
} from '@/features/flow/model/settings-schema.ts'
import { HarborResourceActions } from '@/features/harbor/components/HarborResourceActions'
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { ConfirmDialog } from '@/shared/ui/confirm-dialog'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
// Assuming this exists from your example
import { Textarea } from '@/shared/ui/textarea'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip'

interface FlowSettingsTabProps {
  draft: FlowDraft
}

interface FlowActionRowProps {
  icon: LucideIcon
  label: string
  description: string
  buttonLabel: string
  buttonVariant?: 'destructive' | 'outline' | 'default'
  destructive?: boolean
  disabled?: boolean
  disabledTooltip?: string
  onClick: () => void
}

function FlowActionRow({
  icon: Icon,
  label,
  description,
  buttonLabel,
  buttonVariant = 'outline',
  destructive = false,
  disabled,
  disabledTooltip,
  onClick,
}: FlowActionRowProps) {
  const btn = (
    <Button  variant={buttonVariant} disabled={disabled} onClick={onClick}>
      <Icon className="h-4 w-4" />
      {buttonLabel}
    </Button>
  )

  return (
    <div
      className={
        destructive
          ? 'border-destructive/30 bg-destructive/5 flex flex-wrap items-center justify-between gap-4 rounded-2xl border p-4'
          : 'bg-primary-foreground flex flex-wrap items-center justify-between gap-4 rounded-2xl p-4'
      }
    >
      <div>
        <p className="text-sm font-medium">{label}</p>
        <p className="text-muted-foreground text-sm">{description}</p>
      </div>
      {disabled && disabledTooltip ? (
        <TooltipProvider delayDuration={150}>
          <Tooltip>
            <TooltipTrigger asChild>
              <div>{btn}</div>
            </TooltipTrigger>
            <TooltipContent side="left" className="bg-popover text-popover-foreground border">
              {disabledTooltip}
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      ) : (
        btn
      )}
    </div>
  )
}

export function FlowDetailsSettingsTab({ draft }: FlowSettingsTabProps) {
  const realm = useActiveRealm()
  const navigate = useRealmNavigate()
  const updateMutation = useUpdateFlow(draft.id)
  const deleteFlow = useDeleteFlow(draft.id)
  const [cloneOpen, setCloneOpen] = useState(false)
  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false)

  const isActive = Boolean(draft.active_version)
  const deleteDisabled = draft.built_in || isActive || deleteFlow.isPending
  const deleteTooltip = draft.built_in
    ? 'Built-in flows cannot be deleted.'
    : isActive
      ? 'Active flows cannot be deleted. Bind a different flow to this slot first.'
      : undefined

  const handleDelete = () => {
    deleteFlow.mutate(undefined, {
      onSuccess: () => {
        setConfirmDeleteOpen(false)
        navigate('/flows')
      },
    })
  }

  const form = useForm<FlowSettingsSchema>({
    resolver: zodResolver(flowSettingsSchema),
    defaultValues: {
      name: draft.name || '',
      description: draft.description || '',
    },
  })

  // 1. Sync Form with Draft Data
  // If the draft data refreshes (e.g. after a save or external change), update the form
  useEffect(() => {
    form.reset({
      name: draft.name,
      description: draft.description || '',
    })
  }, [draft, form])

  // 2. Handle Submit
  const onSubmit = (values: FlowSettingsSchema) => {
    updateMutation.mutate(values, {
      onSuccess: () => {
        // Reset form with new values to mark it as "pristine"
        // Adjust 'data' based on what your backend actually returns
        form.reset({
          name: values.name,
          description: values.description,
        })
      },
    })
  }

  // 3. Connect to Floating Action Bar
  // This will show the "Save Changes" / "Discard" bar at the bottom when form is dirty
  useFormPersistence(form, onSubmit, updateMutation.isPending)

  return (
    <div className="grid min-h-full w-full items-start gap-6 p-6 xl:grid-cols-[minmax(0,1fr)_20rem]">
      <div className="min-w-0 space-y-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>General Settings</CardTitle>
                <CardDescription>Manage the basic identity of this flow.</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="bg-primary-foreground space-y-4 rounded-2xl p-4">
                  <FormInput
                    control={form.control}
                    name="name"
                    label="Flow Name"
                    placeholder="e.g. Browser Login"
                    description="A unique name to identify this flow."
                    disabled={draft.built_in} // Optional: Prevent renaming system flows?
                  />

                  <FormField
                    control={form.control}
                    name="description"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Description</FormLabel>
                        <FormControl>
                          <Textarea
                            placeholder="Describe the purpose of this authentication flow..."
                            className="resize-none"
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>visible to other administrators.</FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
              </CardContent>
            </Card>
          </form>
        </Form>

        <Card>
          <CardHeader>
            <CardTitle>Harbor</CardTitle>
            <CardDescription>
              Export this flow as a portable bundle, or import a bundle to replace its
              configuration.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="bg-primary-foreground flex items-center justify-between gap-4 rounded-2xl p-4">
              <p className="text-muted-foreground text-sm">
                Download the flow definition or upload a bundle to apply changes.
              </p>
              {realm ? (
                <HarborResourceActions
                  scope="flow"
                  id={draft.id}
                  resourceLabel={draft.name}
                  invalidateKeys={[
                    ['flows', realm],
                    ['flow-draft', realm, draft.id],
                    ['flow-drafts', realm],
                    ['flow-versions', draft.id],
                  ]}
                />
              ) : null}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Flow Management</CardTitle>
            <CardDescription>Duplicate or permanently remove this flow.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              <FlowActionRow
                icon={Copy}
                label="Duplicate Flow"
                description="Create a new draft with a copy of this flow's configuration, optionally publishing it as active."
                buttonLabel="Duplicate"
                buttonVariant="default"
                onClick={() => setCloneOpen(true)}
              />
              <FlowActionRow
                icon={Trash2}
                label="Delete Flow"
                description="Permanently removes this flow and its versions. Built-in and active flows cannot be deleted."
                buttonLabel="Delete"
                buttonVariant="destructive"
                destructive
                disabled={deleteDisabled}
                disabledTooltip={deleteTooltip}
                onClick={() => setConfirmDeleteOpen(true)}
              />
            </div>
          </CardContent>
        </Card>
      </div>

      <aside className="min-w-0 xl:sticky xl:top-6 xl:self-start">
        <FlowSummaryPanel draft={draft} />
      </aside>

      <CloneFlowDialog draft={draft} open={cloneOpen} onOpenChange={setCloneOpen} />

      <ConfirmDialog
        open={confirmDeleteOpen}
        onOpenChange={setConfirmDeleteOpen}
        title="Delete flow"
        desc={
          <div className="space-y-3">
            <p>
              Are you sure you want to delete <span className="font-medium">{draft.name}</span>?
            </p>
            <p>This permanently removes the flow and all of its versions. This cannot be undone.</p>
          </div>
        }
        confirmText="Delete flow"
        destructive
        isLoading={deleteFlow.isPending}
        handleConfirm={handleDelete}
      />
    </div>
  )
}
