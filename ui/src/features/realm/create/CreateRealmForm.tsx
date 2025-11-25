import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import { z } from 'zod'

import { Button } from '@/components/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/card'
import { Form } from '@/components/form'
import { useCreateRealm } from '@/entities/realm/api/useCreateRealm.ts'
import { FormInput } from '@/shared/ui/form-input.tsx'

const formSchema = z.object({
  name: z
    .string()
    .min(3, { message: 'Realm name must be at least 3 characters.' })
    .max(30, { message: 'Realm name must be less than 30 characters.' })
    .regex(/^[a-z0-9-]+$/, {
      message: 'Only lowercase letters, numbers, and hyphens allowed.',
    }),
})

type FormValues = z.infer<typeof formSchema>

export function CreateRealmForm() {
  const navigate = useNavigate()
  const createRealmMutation = useCreateRealm()
  const { t } = useTranslation('realm')

  const form = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
    },
  })

  const onSubmit = (values: FormValues) => {
    createRealmMutation.mutate(values)
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>{t('FORMS.CREATE_REALM.TITLE')}</CardTitle>
        <CardDescription>{t('FORMS.CREATE_REALM.DESCRIPTION')}</CardDescription>
      </CardHeader>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)}>
          <CardContent className="space-y-4">
            <FormInput
              control={form.control}
              name="name"
              label={t('FORMS.CREATE_REALM.FIELDS.REALM_NAME')}
              placeholder={t('FORMS.CREATE_REALM.FIELDS.REALM_NAME_PLACEHOLDER')}
            />
          </CardContent>
          <CardFooter className="flex justify-between">
            <Button
              type="button"
              variant="outline"
              onClick={() => navigate(-1)} // Go back
              disabled={createRealmMutation.isPending}
            >
              {t('FORMS.CREATE_REALM.CANCEL_BTN')}
            </Button>
            <Button type="submit" disabled={createRealmMutation.isPending}>
              {createRealmMutation.isPending
                ? t('FORMS.CREATE_REALM.CREATE_BTN_LOADING')
                : t('FORMS.CREATE_REALM.CREATE_BTN')}
            </Button>
          </CardFooter>
        </form>
      </Form>
    </Card>
  )
}
