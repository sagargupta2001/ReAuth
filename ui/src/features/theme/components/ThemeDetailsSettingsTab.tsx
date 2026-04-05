import { useEffect, useMemo } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { useUpdateTheme } from '@/features/theme/api/useUpdateTheme'
import { useUpdateThemeFlowBinding } from '@/features/theme/api/useUpdateThemeFlowBinding'
import type { Theme } from '@/entities/theme/model/types'
import { useFlows } from '@/features/flow/api/useFlows'
import {
  type ThemeSettingsSchema,
  themeSettingsSchema,
} from '@/features/theme/model/settings-schema'
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
import { Textarea } from '@/shared/ui/textarea'
import { Button } from '@/components/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/select'

interface ThemeDetailsSettingsTabProps {
  theme: Theme
}

export function ThemeDetailsSettingsTab({ theme }: ThemeDetailsSettingsTabProps) {
  const updateMutation = useUpdateTheme(theme.id)
  const updateFlowBinding = useUpdateThemeFlowBinding(theme.id)
  const { data: flows = [] } = useFlows()
  const flowOptions = useMemo(
    () => flows.filter((flow) => flow.is_draft),
    [flows],
  )
  const boundFlow = useMemo(
    () => flows.find((flow) => flow.id === theme.flow_binding_id) || null,
    [flows, theme.flow_binding_id],
  )
  const boundFlowLabel = boundFlow
    ? boundFlow.alias
    : theme.flow_binding_id
      ? theme.flow_binding_id
      : 'Realm default browser flow'
  const missingBoundFlow = Boolean(theme.flow_binding_id && !boundFlow)

  const form = useForm<ThemeSettingsSchema>({
    resolver: zodResolver(themeSettingsSchema),
    defaultValues: {
      name: theme.name || '',
      description: theme.description || '',
    },
  })

  useEffect(() => {
    form.reset({
      name: theme.name,
      description: theme.description || '',
    })
  }, [theme, form])


  const onSubmit = (values: ThemeSettingsSchema) => {
    updateMutation.mutate(values, {
      onSuccess: () => {
        form.reset({
          name: values.name,
          description: values.description,
        })
      },
    })
  }

  useFormPersistence(form, onSubmit, updateMutation.isPending)

  return (
    <div className="max-w-2xl space-y-6 p-6">
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>General Settings</CardTitle>
              <CardDescription>Manage the basic identity of this theme.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <FormInput
                control={form.control}
                name="name"
                label="Theme Name"
                placeholder="e.g. Brand Refresh"
                description="A unique name to identify this theme."
              />

              <FormField
                control={form.control}
                name="description"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Description</FormLabel>
                    <FormControl>
                      <Textarea
                        placeholder="Describe the purpose of this theme..."
                        className="resize-none"
                        {...field}
                      />
                    </FormControl>
                    <FormDescription>Visible to other administrators.</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </CardContent>
          </Card>
        </form>
      </Form>

      <Card>
        <CardHeader>
          <CardTitle>Flow Binding</CardTitle>
          <CardDescription>
            Link this theme to a flow draft so Action Binder suggestions match the flow graph.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs font-semibold">Bound Flow</span>
              {theme.flow_binding_id && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2 text-[10px]"
                  onClick={() => updateFlowBinding.mutate(null)}
                  disabled={updateFlowBinding.isPending}
                >
                  Clear binding
                </Button>
              )}
            </div>
            <Select
              value={theme.flow_binding_id ?? 'none'}
              onValueChange={(value) => {
                if (value === 'none') {
                  updateFlowBinding.mutate(null)
                  return
                }
                updateFlowBinding.mutate(value)
              }}
              disabled={updateFlowBinding.isPending}
            >
              <SelectTrigger className="h-8 text-xs">
                <SelectValue placeholder="Select flow draft" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">No binding (use realm default)</SelectItem>
                {missingBoundFlow && (
                  <SelectItem value={theme.flow_binding_id as string}>
                    Unknown flow ({theme.flow_binding_id})
                  </SelectItem>
                )}
                {flowOptions.length === 0 ? (
                  <SelectItem value="empty" disabled>
                    No flow drafts available
                  </SelectItem>
                ) : (
                  flowOptions.map((flow) => (
                    <SelectItem key={flow.id} value={flow.id}>
                      {flow.alias}
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
            <p className="text-muted-foreground text-[10px]">
              Current: {boundFlowLabel}
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Metadata</CardTitle>
          <CardDescription>Read-only details about this theme.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 text-sm">
          <div>
            <p className="text-muted-foreground text-xs">Theme ID</p>
            <p className="font-mono text-xs">{theme.id}</p>
          </div>
          <div>
            <p className="text-muted-foreground text-xs">Created</p>
            <p>{new Date(theme.created_at).toLocaleString()}</p>
          </div>
          <div>
            <p className="text-muted-foreground text-xs">Last Updated</p>
            <p>{new Date(theme.updated_at).toLocaleString()}</p>
          </div>
        </CardContent>
      </Card>

    </div>
  )
}
