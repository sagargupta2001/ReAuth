import { useTranslation } from 'react-i18next'

import { Badge } from '@/shared/ui/badge.tsx'
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from '@/shared/ui/sheet.tsx'
import type { LogEntry } from '@/entities/log/model/types.ts'

interface Props {
  log: LogEntry | null
  onOpenChange: (open: boolean) => void
}

// Helper to determine badge color
function getLevelVariant(level: string): 'default' | 'secondary' | 'destructive' {
  switch (level) {
    case 'ERROR':
      return 'destructive'
    case 'WARN':
      return 'secondary'
    case 'INFO':
      return 'default'
    default:
      return 'secondary'
  }
}

export function LogRowDetail({ log, onOpenChange }: Props) {
  const { t } = useTranslation('logs')

  return (
    <Sheet open={!!log} onOpenChange={onOpenChange}>
      <SheetContent className="w-[500px] overflow-y-auto sm:max-w-xl">
        <SheetHeader>
          <SheetTitle className="text-left">{t('LOG_ROW_DETAIL.TITLE')}</SheetTitle>
          <SheetDescription className="text-left">
            {t('LOG_ROW_DETAIL.DESCRIPTION')}
          </SheetDescription>
        </SheetHeader>
        {log && (
          <div className="mt-4 space-y-4">
            <div className="space-y-1">
              <h3 className="text-muted-foreground text-xs font-medium">
                {t('LOG_ROW_DETAIL.TIMESTAMP_LABEL')}
              </h3>
              <p>{new Date(log.timestamp).toISOString()}</p>
            </div>
            <div className="space-y-1">
              <h3 className="text-muted-foreground text-xs font-medium">
                {t('LOG_ROW_DETAIL.LEVEL_LABEL')}
              </h3>
              <Badge variant={getLevelVariant(log.level)}>{log.level}</Badge>
            </div>
            <div className="space-y-1">
              <h3 className="text-muted-foreground text-xs font-medium">
                {t('LOG_ROW_DETAIL.TARGET_LABEL')}
              </h3>
              <p className="font-mono text-sm">{log.target}</p>
            </div>
            <div className="space-y-1">
              <h3 className="text-muted-foreground text-xs font-medium">
                {t('LOG_ROW_DETAIL.MESSAGE_LABEL')}
              </h3>
              <p className="font-medium">{log.message || '---'}</p>
            </div>
            {Object.keys(log.fields).length > 0 && (
              <div>
                <h3 className="text-muted-foreground text-xs font-medium">
                  {t('LOG_ROW_DETAIL.FIELDS_LABEL')}
                </h3>
                <pre className="bg-muted mt-2 w-full rounded-md p-4 text-xs">
                  {JSON.stringify(log.fields, null, 2)}
                </pre>
              </div>
            )}
          </div>
        )}
      </SheetContent>
    </Sheet>
  )
}
