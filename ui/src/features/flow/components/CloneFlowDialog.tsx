import { useEffect } from 'react'

import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { z } from 'zod'

import { Button } from '@/components/button'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '@/components/dialog'
import type { FlowDraft } from '@/entities/flow/model/types'
import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useCloneFlow } from '@/features/flow/api/useCloneFlow'
import { Checkbox } from '@/shared/ui/checkbox'
import { Form } from '@/shared/ui/form'
import { FormInput } from '@/shared/ui/form-input'
import { Separator } from '@/shared/ui/separator'

interface Props {
  draft: FlowDraft
  open: boolean
  onOpenChange: (open: boolean) => void
}

const cloneFlowSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  make_active: z.boolean(),
})

type FormData = z.infer<typeof cloneFlowSchema>

export function CloneFlowDialog({ draft, open, onOpenChange }: Props) {
  const navigate = useRealmNavigate()
  const cloneFlow = useCloneFlow(draft.id)

  const form = useForm<FormData>({
    resolver: zodResolver(cloneFlowSchema),
    defaultValues: { name: `Copy of ${draft.name}`, make_active: false },
  })

  // Re-seed the suggested name whenever the dialog reopens for a (possibly different) flow.
  useEffect(() => {
    if (open) form.reset({ name: `Copy of ${draft.name}`, make_active: false })
  }, [open, draft.name, form])

  const handleOpenChange = (newOpen: boolean) => {
    onOpenChange(newOpen)
    if (!newOpen) form.reset()
  }

  const onSubmit = (values: FormData) => {
    cloneFlow.mutate(values, {
      onSuccess: (newDraft) => {
        handleOpenChange(false)
        navigate(`/flows/${newDraft.id}/builder`)
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>Duplicate flow</DialogTitle>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <div className="grid gap-4 px-6 pb-6">
            <FormInput
              control={form.control}
              name="name"
              label="New flow name"
              placeholder="e.g., Copy of Browser Login"
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
                  Publishes the duplicate immediately and binds it to its flow-type slot, replacing
                  the currently active {draft.flow_type} flow.
                </p>
              </div>
            </div>
          </div>

          <DialogFooter className="gap-1 py-3 pr-3">
            <Button variant="outline" type="button" onClick={() => handleOpenChange(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={form.handleSubmit(onSubmit)} disabled={cloneFlow.isPending}>
              {cloneFlow.isPending ? 'Duplicating...' : 'Duplicate Flow'}
            </Button>
          </DialogFooter>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
