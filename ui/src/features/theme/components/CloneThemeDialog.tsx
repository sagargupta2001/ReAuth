import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/dialog'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import type { Theme } from '@/entities/theme/model/types'
import { useCloneTheme } from '@/features/theme/api/useCloneTheme'
import { Checkbox } from '@/shared/ui/checkbox'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Separator } from '@/shared/ui/separator'

interface Props {
  theme: Theme
  open: boolean
  onOpenChange: (open: boolean) => void
}

const cloneThemeSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  make_active: z.boolean(),
})

type FormData = z.infer<typeof cloneThemeSchema>

export function CloneThemeDialog({ theme, open, onOpenChange }: Props) {
  const navigate = useRealmNavigate()
  const cloneTheme = useCloneTheme(theme.id)

  const form = useForm<FormData>({
    resolver: zodResolver(cloneThemeSchema),
    defaultValues: { name: `Copy of ${theme.name}`, make_active: false },
  })

  // Re-seed the suggested name whenever the dialog reopens for a (possibly different) theme.
  useEffect(() => {
    if (open) form.reset({ name: `Copy of ${theme.name}`, make_active: false })
  }, [open, theme.name, form])

  const handleOpenChange = (newOpen: boolean) => {
    onOpenChange(newOpen)
    if (!newOpen) form.reset()
  }

  const onSubmit = (values: FormData) => {
    cloneTheme.mutate(values, {
      onSuccess: (created) => {
        handleOpenChange(false)
        navigate(`/themes/${created.theme.id}`)
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Duplicate theme</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
            <FormInput
              control={form.control}
              name="name"
              label="New theme name"
              placeholder="e.g., Copy of Brand Refresh"
            />

            <div className="mt-2 flex items-start space-x-3">
              <Checkbox
                id="make_active"
                checked={form.watch('make_active')}
                onCheckedChange={(checked) => form.setValue('make_active', checked as boolean)}
              />
              <div className="-mt-0.5 grid gap-1.5 leading-none">
                <label
                  htmlFor="make_active"
                  className="cursor-pointer text-[14px] leading-none font-medium"
                >
                  and make it active
                </label>
                <p className="text-muted-foreground mt-1 text-[13px]">
                  Publishes the duplicate immediately and makes it the realm&apos;s active theme,
                  replacing the current default.
                </p>
              </div>
            </div>
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={form.handleSubmit(onSubmit)} disabled={cloneTheme.isPending}>
              {cloneTheme.isPending ? 'Duplicating...' : 'Duplicate Theme'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
