import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Input } from '@/components/input'
import { Label } from '@/components/label'
import { Textarea } from '@/components/textarea'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useCreateTheme } from '@/features/theme/api/useCreateTheme'

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

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormData>({
    resolver: zodResolver(createThemeSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  })

  const onSubmit = async (data: FormData) => {
    try {
      const created = await createTheme(data)
      onOpenChange(false)
      reset()
      navigate(`/themes/${created.theme.id}`)
    } catch (error) {
      console.error('Failed to create theme', error)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Theme</DialogTitle>
          <DialogDescription>
            Start a new visual experience for your authentication flows.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="name">Theme Name</Label>
            <Input id="name" placeholder="e.g., Brand Refresh" {...register('name')} />
            {errors.name && <p className="text-destructive text-xs">{errors.name.message}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">Description (Optional)</Label>
            <Textarea
              id="description"
              placeholder="Describe the purpose of this theme..."
              {...register('description')}
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Create Theme
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
