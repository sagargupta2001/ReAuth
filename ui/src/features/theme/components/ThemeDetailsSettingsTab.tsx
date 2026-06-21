import { useEffect, useState } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { AlertTriangle, Copy, Trash2 } from 'lucide-react'
import { useForm } from 'react-hook-form'

import { Button } from '@/components/button'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import type { Theme } from '@/entities/theme/model/types'
import { HarborResourceActions } from '@/features/harbor/components/HarborResourceActions'
import { useDeleteTheme } from '@/features/theme/api/useDeleteTheme'
import { useTheme } from '@/features/theme/api/useTheme'
import { useUpdateTheme } from '@/features/theme/api/useUpdateTheme'
import { CloneThemeDialog } from '@/features/theme/components/CloneThemeDialog'
import {
  type ThemeSettingsSchema,
  themeSettingsSchema,
} from '@/features/theme/model/settings-schema'
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
import { Textarea } from '@/shared/ui/textarea'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/shared/ui/tooltip'

interface ThemeDetailsSettingsTabProps {
  theme: Theme
}

export function ThemeDetailsSettingsTab({ theme }: ThemeDetailsSettingsTabProps) {
  const realm = useActiveRealm()
  const navigate = useRealmNavigate()
  const updateMutation = useUpdateTheme(theme.id)
  const deleteTheme = useDeleteTheme(theme.id)
  const { data: details } = useTheme(theme.id)
  const [cloneOpen, setCloneOpen] = useState(false)
  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false)

  const isActive = typeof details?.active_version_number === 'number'
  const deleteDisabled = theme.is_system || isActive || deleteTheme.isPending
  const deleteTooltip = theme.is_system
    ? 'The default theme cannot be deleted.'
    : isActive
      ? 'The active theme cannot be deleted. Activate a different theme first.'
      : undefined

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

  const handleDelete = () => {
    deleteTheme.mutate(undefined, {
      onSuccess: () => {
        setConfirmDeleteOpen(false)
        navigate('/themes')
      },
    })
  }

  return (
    <>
      <div className="space-y-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>General Settings</CardTitle>
                <CardDescription>Manage the basic identity of this theme.</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="bg-primary-foreground space-y-4 rounded-2xl p-4">
                  <FormInput
                    control={form.control}
                    name="name"
                    label="Theme Name"
                    placeholder="e.g. Brand Refresh"
                    description="A unique name to identify this theme."
                    disabled={theme.is_system}
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
                </div>
              </CardContent>
            </Card>
          </form>
        </Form>

        <Card>
          <CardHeader>
            <CardTitle>Harbor</CardTitle>
            <CardDescription>
              Export this theme as a portable bundle, or import a bundle to replace its
              configuration.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="bg-primary-foreground flex items-center justify-between gap-4 rounded-2xl p-4">
              <p className="text-muted-foreground text-sm">
                Download the theme definition or upload a bundle to apply changes.
              </p>
              {realm ? (
                <HarborResourceActions
                  scope="theme"
                  id={theme.id}
                  resourceLabel={theme.name}
                  invalidateKeys={[
                    ['themes', realm],
                    ['themes', realm, theme.id],
                    ['themes', realm, theme.id, 'draft'],
                    ['themes', realm, theme.id, 'assets'],
                    ['themes', realm, theme.id, 'versions'],
                    ['theme-bindings', realm, theme.id],
                    ['theme-preview', realm, theme.id],
                  ]}
                />
              ) : null}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Theme Management</CardTitle>
            <CardDescription>Duplicate this theme into a new draft.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="bg-primary-foreground flex flex-wrap items-center justify-between gap-4 rounded-2xl p-4">
              <div>
                <p className="text-sm font-medium">Duplicate Theme</p>
                <p className="text-muted-foreground text-sm">
                  Create a new theme with a copy of this theme&apos;s tokens, layout, and pages,
                  optionally making it active.
                </p>
              </div>
              <Button type="button" className="gap-2" onClick={() => setCloneOpen(true)}>
                <Copy className="h-4 w-4" />
                Duplicate
              </Button>
            </div>
          </CardContent>
        </Card>

        <div className="border-destructive/50 bg-destructive/10 rounded-xl border p-4">
          <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
            <div className="flex items-start gap-3">
              <div className="bg-destructive/20 text-destructive rounded-full p-2">
                <AlertTriangle className="h-4 w-4" />
              </div>
              <div>
                <div className="text-destructive text-sm font-semibold">Danger Zone</div>
                <p className="text-muted-foreground text-xs">
                  Permanently removes this theme and all of its versions. The default theme and the
                  active theme cannot be deleted.
                </p>
              </div>
            </div>
            {deleteDisabled && deleteTooltip ? (
              <TooltipProvider delayDuration={150}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <div>
                      <Button variant="destructive" className="gap-2" disabled>
                        <Trash2 className="h-4 w-4" />
                        Delete Theme
                      </Button>
                    </div>
                  </TooltipTrigger>
                  <TooltipContent side="left" className="bg-popover text-popover-foreground border">
                    {deleteTooltip}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            ) : (
              <Button
                variant="destructive"
                className="gap-2"
                disabled={deleteDisabled}
                onClick={() => setConfirmDeleteOpen(true)}
              >
                <Trash2 className="h-4 w-4" />
                Delete Theme
              </Button>
            )}
          </div>
        </div>
      </div>

      <CloneThemeDialog theme={theme} open={cloneOpen} onOpenChange={setCloneOpen} />

      <ConfirmDialog
        open={confirmDeleteOpen}
        onOpenChange={setConfirmDeleteOpen}
        title="Delete theme"
        desc={
          <div className="space-y-3">
            <p>
              Are you sure you want to delete <span className="font-medium">{theme.name}</span>?
            </p>
            <p>
              This permanently removes the theme and all of its versions. This cannot be undone.
            </p>
          </div>
        }
        confirmText="Delete theme"
        destructive
        isLoading={deleteTheme.isPending}
        handleConfirm={handleDelete}
      />
    </>
  )
}
