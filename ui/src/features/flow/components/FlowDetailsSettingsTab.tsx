import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { FlowDraft } from '@/entities/flow/model/types'
import { HarborResourceActions } from '@/features/harbor/components/HarborResourceActions'
import { useUpdateFlow } from '@/features/flow/api/useUpdateFlow.ts'
import {
  type FlowSettingsSchema,
  flowSettingsSchema,
} from '@/features/flow/model/settings-schema.ts'
// Shadcn Textarea
import { useFormPersistence } from '@/shared/hooks/useFormPersistence'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
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

interface FlowSettingsTabProps {
  draft: FlowDraft
}

export function FlowDetailsSettingsTab({ draft }: FlowSettingsTabProps) {
  const realm = useActiveRealm()
  const updateMutation = useUpdateFlow(draft.id)

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
    <div className="max-w-2xl space-y-6 p-6">
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
          <CardTitle>Import / Export</CardTitle>
          <CardDescription>
            Export this flow as a portable bundle, or import a bundle to replace its configuration.
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
    </div>
  )
}
