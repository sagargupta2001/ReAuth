import { useTranslation } from 'react-i18next'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/card'
import { AuthFlowExecutor } from '@/features/auth/components/AuthFlowExecutor.tsx'

export function LoginPage() {
  const { t } = useTranslation('common')

  return (
    <Card className="gap-4">
      <CardHeader>
        <CardTitle className="text-lg tracking-tight">{t('LOGIN_PAGE.TITLE')}</CardTitle>
        <CardDescription>{t('LOGIN_PAGE.DESCRIPTION')}</CardDescription>
      </CardHeader>
      <CardContent>
        <AuthFlowExecutor />
      </CardContent>
    </Card>
  )
}
