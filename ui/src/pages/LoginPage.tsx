import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router-dom'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { LoginForm } from '@/features/auth/components/LoginForm.tsx'

export function LoginPage() {
  const [searchParams] = useSearchParams()
  const redirect = searchParams.get('redirect') ?? undefined
  const { t } = useTranslation('common')

  return (
    <Card className="gap-4">
      <CardHeader>
        <CardTitle className="text-lg tracking-tight">{t('LOGIN_PAGE.TITLE')}</CardTitle>
        <CardDescription>{t('LOGIN_PAGE.DESCRIPTION')}</CardDescription>
      </CardHeader>
      <CardContent>
        <LoginForm redirectTo={redirect} />
      </CardContent>
    </Card>
  )
}
