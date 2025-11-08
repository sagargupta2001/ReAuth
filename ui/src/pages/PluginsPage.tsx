import { type ChangeEvent, useMemo, useState } from 'react'

import { ArrowDownAZ, ArrowUpAZ, SlidersHorizontal } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useSearchParams } from 'react-router-dom'

import { IconDiscord } from '@/assets/brand-icons/icon-discord.tsx'
import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Separator } from '@/components/separator'
import { usePlugins } from '@/entities/plugin/api/usePlugins.ts'
import { Search } from '@/features/Search/components/Search'
import { ThemeSwitch } from '@/features/ThemeSwitch/ThemeSwitch'
import { ProfileDropdown } from '@/features/auth/ProfileDropdown'
import { ConfigDrawer } from '@/widgets/ConfigDrawer/ConfigDrawer'
import { Main } from '@/widgets/Layout/Main'
import { Header } from '@/widgets/Layout/components/header'

type PluginType = 'all' | 'connected' | 'notConnected'

const pluginText = new Map<PluginType, string>([
  ['all', 'All Plugins'],
  ['connected', 'Connected'],
  ['notConnected', 'Not Connected'],
])

export function PluginsPage() {
  const { t } = useTranslation('plugins')
  const [searchParams, setSearchParams] = useSearchParams()

  // Read from URL params with defaults
  const initialFilter = searchParams.get('filter') || ''
  const initialType = (searchParams.get('type') as PluginType) || 'all'
  const initialSort = (searchParams.get('sort') as 'asc' | 'desc') || 'asc'

  const [searchTerm, setSearchTerm] = useState(initialFilter)
  const [appType, setAppType] = useState<PluginType>(initialType)
  const [sort, setSort] = useState<'asc' | 'desc'>(initialSort)

  const navigate = useNavigate()
  const { data, isLoading: isPluginsLoading } = usePlugins()

  const plugins = useMemo(() => data?.manifests, [data, isPluginsLoading])

  const filteredPlugins = useMemo(() => {
    if (!plugins) return []
    return plugins
      .sort((a, b) =>
        sort === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name),
      )
      .filter((app) => app.name.toLowerCase().includes(searchTerm.toLowerCase()))
  }, [plugins, sort, appType, searchTerm, isPluginsLoading])

  const updateSearchParams = (updates: Record<string, string | undefined>) => {
    const params = new URLSearchParams(searchParams)
    Object.entries(updates).forEach(([key, value]) => {
      if (value === undefined) params.delete(key)
      else params.set(key, value)
    })
    setSearchParams(params)
  }

  const handleSearch = (e: ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setSearchTerm(value)
    updateSearchParams({ filter: value || undefined })
  }

  const handleTypeChange = (value: PluginType) => {
    setAppType(value)
    updateSearchParams({ type: value === 'all' ? undefined : value })
  }

  const handleSortChange = (value: 'asc' | 'desc') => {
    setSort(value)
    updateSearchParams({ sort: value })
  }

  return (
    <>
      {/* ===== Top Heading ===== */}
      <Header>
        <Search />
        <div className="ms-auto flex items-center gap-4">
          <ThemeSwitch />
          <ConfigDrawer />
          <ProfileDropdown />
        </div>
      </Header>

      {/* ===== Content ===== */}
      <Main fixed>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('TITLE')}</h1>
          <p className="text-muted-foreground">{t('SUB_TITLE')}</p>
        </div>

        <div className="my-4 flex items-end justify-between sm:my-0 sm:items-center">
          <div className="flex flex-col gap-4 sm:my-4 sm:flex-row">
            <Input
              placeholder="Filter plugins..."
              className="h-9 w-40 lg:w-[250px]"
              value={searchTerm}
              onChange={handleSearch}
            />
            <Select value={appType} onValueChange={handleTypeChange}>
              <SelectTrigger className="w-36">
                <SelectValue>{pluginText.get(appType)}</SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Plugins</SelectItem>
                <SelectItem value="connected">Connected</SelectItem>
                <SelectItem value="notConnected">Not Connected</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Select value={sort} onValueChange={handleSortChange}>
            <SelectTrigger className="w-16">
              <SelectValue>
                <SlidersHorizontal size={18} />
              </SelectValue>
            </SelectTrigger>
            <SelectContent align="end">
              <SelectItem value="asc">
                <div className="flex items-center gap-4">
                  <ArrowUpAZ size={16} />
                  <span>Ascending</span>
                </div>
              </SelectItem>
              <SelectItem value="desc">
                <div className="flex items-center gap-4">
                  <ArrowDownAZ size={16} />
                  <span>Descending</span>
                </div>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <Separator className="shadow-sm" />

        <ul className="faded-bottom no-scrollbar grid gap-4 overflow-auto pt-4 pb-16 md:grid-cols-2 lg:grid-cols-3">
          {filteredPlugins.map((plugin) => (
            <li key={plugin.name} className="rounded-lg border p-4 hover:shadow-md">
              <div className="mb-8 flex items-center justify-between">
                <div className="bg-muted flex size-10 items-center justify-center rounded-lg p-2">
                  <IconDiscord />
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  className={`${'border border-blue-300 bg-blue-50 hover:bg-blue-100 dark:border-blue-700 dark:bg-blue-950 dark:hover:bg-blue-900'}`}
                  onClick={() => navigate(plugin.frontend.route)}
                >
                  {'Connected'}
                </Button>
              </div>
              <div>
                <h2 className="mb-1 font-semibold">{plugin.name}</h2>
                <p className="line-clamp-2 text-gray-500">{plugin.name}</p>
              </div>
            </li>
          ))}
        </ul>
      </Main>
    </>
  )
}
