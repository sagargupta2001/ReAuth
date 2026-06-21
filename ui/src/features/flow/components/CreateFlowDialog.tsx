import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useCreateDraft } from '@/features/flow-builder/api/useCreateDraft'
import { Button } from '@/components/button'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select.tsx'
import { Separator } from '@/shared/ui/separator'
import { Textarea } from '@/shared/ui/textarea'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const FLOW_TYPES = [
  { value: 'browser', label: 'Browser Login' },
  { value: 'registration', label: 'Registration' },
  { value: 'direct', label: 'Direct Grant' },
  { value: 'reset', label: 'Reset Credentials' },
  { value: 'invitation', label: 'Invitation' },
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

  const form = useForm<FormData>({
    resolver: zodResolver(createFlowSchema),
    defaultValues: {
      name: '',
      description: '',
      flow_type: '',
    },
  })

  const handleOpenChange = (newOpen: boolean) => {
    onOpenChange(newOpen)
    if (!newOpen) form.reset()
  }

  const onSubmit = async (data: FormData) => {
    try {
      const newDraft = await createDraft(data)
      handleOpenChange(false)
      navigate(`/flows/${newDraft.id}/builder`)
    } catch (error) {
      console.error('Failed to create flow', error)
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Create new flow</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
            <FormInput
              control={form.control}
              name="name"
              label="Flow Name"
              placeholder="e.g., Partner Login Flow"
            />

            <FormField
              control={form.control}
              name="flow_type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Flow Type</FormLabel>
                  <Select onValueChange={field.onChange} value={field.value}>
                    <FormControl>
                      <SelectTrigger>
                        <SelectValue placeholder="Select flow type" />
                      </SelectTrigger>
                    </FormControl>
                    <SelectContent>
                      {FLOW_TYPES.map((type) => (
                        <SelectItem key={type.value} value={type.value}>
                          {type.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Description (Optional)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Briefly describe what this flow does..."
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
              {isPending ? 'Creating...' : 'Create Flow'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
