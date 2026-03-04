import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'

import { useUpdateTheme } from '@/features/theme/api/useUpdateTheme'
import type { Theme } from '@/entities/theme/model/types'
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

interface ThemeDetailsSettingsTabProps {
  theme: Theme
}

export function ThemeDetailsSettingsTab({ theme }: ThemeDetailsSettingsTabProps) {
  const updateMutation = useUpdateTheme(theme.id)

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
