import { useTranslation } from 'react-i18next'

import { Search } from '@/features/Search/components/Search'
import { ThemeSwitch } from '@/features/ThemeSwitch/ThemeSwitch'
import { ProfileDropdown } from '@/features/auth/ProfileDropdown'
import { ConfigDrawer } from '@/widgets/ConfigDrawer/ConfigDrawer'
import { Main } from '@/widgets/Layout/Main'
import { Header } from '@/widgets/Layout/components/header'
import { PluginList } from '@/widgets/PluginList/PluginList'

export function PluginsPage() {
  const { t } = useTranslation('plugins')

  return (
    <>
      <Header>
        <Search />
        <div className="ms-auto flex items-center gap-4">
          <ThemeSwitch />
          <ConfigDrawer />
          <ProfileDropdown />
        </div>
      </Header>

      <Main fixed>
        <div className="mb-6">
          <h1 className="text-2xl font-bold tracking-tight">{t('TITLE')}</h1>
          <p className="text-muted-foreground">{t('SUB_TITLE')}</p>
        </div>

        <PluginList />
      </Main>
    </>
  )
}
