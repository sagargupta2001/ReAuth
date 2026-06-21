import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/dialog'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useCreateTheme } from '@/features/theme/api/useCreateTheme'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Separator } from '@/shared/ui/separator'
import { Textarea } from '@/shared/ui/textarea'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const createThemeSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  description: z.string().optional(),
})

type FormData = z.infer<typeof createThemeSchema>

export function CreateThemeDialog({ open, onOpenChange }: Props) {
  const navigate = useRealmNavigate()
  const { mutateAsync: createTheme, isPending } = useCreateTheme()

  const form = useForm<FormData>({
    resolver: zodResolver(createThemeSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const handleOpenChange = (newOpen: boolean) => {
    onOpenChange(newOpen)
    if (!newOpen) form.reset()
  }

  const onSubmit = async (data: FormData) => {
    try {
      const created = await createTheme(data)
      handleOpenChange(false)
      navigate(`/themes/${created.theme.id}`)
    } catch (error) {
      console.error('Failed to create theme', error)
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Create new theme</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
            <FormInput
              control={form.control}
              name="name"
              label="Theme Name"
              placeholder="e.g., Brand Refresh"
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Description (Optional)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Describe the purpose of this theme..."
                      className="resize-none"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={form.handleSubmit(onSubmit)} disabled={isPending}>
              {isPending ? 'Creating...' : 'Create Theme'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
