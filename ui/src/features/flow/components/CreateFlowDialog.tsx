import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2 } from 'lucide-react'
import { Controller, useForm } from 'react-hook-form'
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
import { useCreateDraft } from '@/features/flow-builder/api/useCreateDraft'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select.tsx'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const FLOW_TYPES = [
  { value: 'browser', label: 'Browser Login' },
  { value: 'registration', label: 'Registration' },
  { value: 'direct', label: 'Direct Grant' },
  { value: 'reset', label: 'Reset Credentials' },
]

const createFlowSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  flow_type: z.string().min(1, 'Flow type is required'),
  description: z.string().optional(),
})

type FormData = z.infer<typeof createFlowSchema>

export function CreateFlowDialog({ open, onOpenChange }: Props) {
  const navigate = useRealmNavigate()
  const { mutateAsync: createDraft, isPending } = useCreateDraft()

  const {
    control,
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormData>({
    resolver: zodResolver(createFlowSchema),
    defaultValues: {
      name: '',
      description: '',
      flow_type: '',
    },
  })

  const onSubmit = async (data: FormData) => {
    try {
      const newDraft = await createDraft(data)
      onOpenChange(false)
      reset()
      navigate(`/flows/${newDraft.id}/builder`)
    } catch (error) {
      console.error('Failed to create flow', error)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create New Flow</DialogTitle>
          <DialogDescription>
            Give your new authentication flow a name to get started.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4 py-4">
          {/* Name */}
          <div className="space-y-2">
            <Label htmlFor="name">Flow Name</Label>
            <Input id="name" placeholder="e.g., Partner Login Flow" {...register('name')} />
            {errors.name && <p className="text-destructive text-xs">{errors.name.message}</p>}
          </div>

          {/* Flow Type */}
          <div className="space-y-2">
            <Label>Flow Type</Label>
            <Controller
              control={control}
              name="flow_type"
              render={({ field }) => (
                <Select onValueChange={field.onChange} value={field.value}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select flow type" />
                  </SelectTrigger>
                  <SelectContent>
                    {FLOW_TYPES.map((type) => (
                      <SelectItem key={type.value} value={type.value}>
                        {type.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            />
            {errors.flow_type && (
              <p className="text-destructive text-xs">{errors.flow_type.message}</p>
            )}
          </div>

          {/* Description */}
          <div className="space-y-2">
            <Label htmlFor="description">Description (Optional)</Label>
            <Textarea
              id="description"
              placeholder="Briefly describe what this flow does..."
              {...register('description')}
            />
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={isPending}>
              {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Create
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
