import { zodResolver } from '@hookform/resolvers/zod'
import { Loader2, LogIn } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'

import { Button } from '@/components/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/form'
import { PasswordInput } from '@/components/password-input'
import { type LoginSchema, loginSchema } from '@/features/auth/schema/loginSchema'
import { FormInput } from '@/shared/ui/form-input.tsx'

import type { AuthScreenProps } from './types'

export function UsernamePasswordScreen({ onSubmit, isLoading, error }: AuthScreenProps) {
  const { t } = useTranslation('common')

  const form = useForm<LoginSchema>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: '', password: '' },
  })

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="grid gap-3">
        {error && <div className="text-destructive mb-2 text-sm">{error}</div>}

        <FormInput
          control={form.control}
          name="username"
          label={t('LOGIN_PAGE.FIELDS.EMAIL')}
          placeholder={t('LOGIN_PAGE.FIELDS.EMAIL_PLACEHOLDER')}
        />

        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem className="relative">
              <FormLabel>{t('LOGIN_PAGE.FIELDS.PASSWORD')}</FormLabel>
              <FormControl>
                <PasswordInput
                  placeholder={t('LOGIN_PAGE.FIELDS.PASSWORD_PLACEHOLDER')}
                  {...field}
                />
              </FormControl>
              <FormMessage />
              <Link
                to="/forgot-password"
                className="text-muted-foreground absolute end-0 -top-0.5 text-sm font-medium hover:opacity-75"
              >
                {t('LOGIN_PAGE.FORGOT_PASSWORD_LINK')}
              </Link>
            </FormItem>
          )}
        />

        <Button className="mt-2" disabled={isLoading}>
          {isLoading ? <Loader2 className="mr-2 animate-spin" /> : <LogIn className="mr-2" />}
          {t('LOGIN_PAGE.SIGN_IN_BTN')}
        </Button>
      </form>
    </Form>
  )
}
