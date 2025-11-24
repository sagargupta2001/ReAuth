import { useTranslation } from 'react-i18next'

import { Main } from '@/widgets/Layout/Main'
import { LogViewerWidget } from '@/widgets/LogViewer/LogViewerWidget.tsx'

export function LogsPage() {
  const { t } = useTranslation('logs')

  return (
    // 1. H-full is crucial here
    <Main fixed className="flex h-full flex-col">
      <div className="mb-4 shrink-0">
        <h1 className="text-2xl font-bold tracking-tight">{t('TITLE')}</h1>
        <p className="text-muted-foreground">{t('DESCRIPTION')}</p>
      </div>

      <div className="bg-background relative min-h-0 flex-1 rounded-md border">
        <LogViewerWidget />
      </div>
    </Main>
  )
}
