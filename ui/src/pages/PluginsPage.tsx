import { useTranslation } from 'react-i18next'

import { Main } from '@/widgets/Layout/Main'
import { PluginList } from '@/features/plugin/components/PluginList.tsx'

export function PluginsPage() {
  const { t } = useTranslation('plugins')

  return (
    <Main className='p-12' fixed>
      <div className="mb-6">
        <h1 className="text-2xl font-bold tracking-tight">{t('TITLE')}</h1>
        <p className="text-muted-foreground">{t('SUB_TITLE')}</p>
      </div>

      <PluginList />
    </Main>
  )
}
