import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/dialog'
import { Form } from '@/components/form'
import { FormInput } from '@/components/form-input'
import { Separator } from '@/components/separator'
import { useCreateRealm } from '@/features/realm/api/useCreateRealm.ts'
import { type FormValues, formSchema } from '@/features/realm/schema/realm.schema.ts'

interface CreateRealmDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateRealmDialog({ open, onOpenChange }: CreateRealmDialogProps) {
  const createRealmMutation = useCreateRealm()
  const { t } = useTranslation('realm')

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: { name: '' },
  })

  const onSubmit = async (values: FormValues) => {
    try {
      // On success the mutation invalidates the realm list and navigates to the new realm.
      await createRealmMutation.mutateAsync(values)
      form.reset()
      onOpenChange(false)
    } catch {
      // Error is surfaced via the mutation's onError toast.
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) form.reset()
        onOpenChange(next)
      }}
    >
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader className="pt-6 pl-6">
          <DialogTitle>{t('FORMS.CREATE_REALM.TITLE')}</DialogTitle>
          <DialogDescription>{t('FORMS.CREATE_REALM.DESCRIPTION')}</DialogDescription>
        </DialogHeader>

        <Separator className="my-1" />

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)}>
            <div className="grid gap-4 px-6 pb-6">
              <FormInput
                control={form.control}
                name="name"
                label={t('FORMS.CREATE_REALM.FIELDS.REALM_NAME')}
                placeholder={t('FORMS.CREATE_REALM.FIELDS.REALM_NAME_PLACEHOLDER')}
              />
            </div>
            <DialogFooter className="gap-1 py-3 pr-3">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
                disabled={createRealmMutation.isPending}
              >
                {t('FORMS.CREATE_REALM.CANCEL_BTN')}
              </Button>
              <Button size="sm" type="submit" disabled={createRealmMutation.isPending}>
                {createRealmMutation.isPending
                  ? t('FORMS.CREATE_REALM.CREATE_BTN_LOADING')
                  : t('FORMS.CREATE_REALM.CREATE_BTN')}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  )
}
