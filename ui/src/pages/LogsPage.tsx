import { useTranslation } from 'react-i18next'

import { Main } from '@/widgets/Layout/Main'
import { LogViewerWidget } from '@/widgets/LogViewer/LogViewerWidget.tsx'

export function LogsPage() {
  const { t } = useTranslation('log-and-analytics')

  return (
    <Main fixed>
      <div className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight">{t('TITLE')}</h1>
        <p className="text-muted-foreground"></p>
      </div>
      <LogViewerWidget />
    </Main>
  )
}
