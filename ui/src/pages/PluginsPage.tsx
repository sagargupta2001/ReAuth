import { type ChangeEvent, useMemo, useState } from 'react'

import { ArrowDownAZ, ArrowUpAZ, Package, SlidersHorizontal } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Link, useSearchParams } from 'react-router-dom'

import { Button } from '@/components/button'
import { Input } from '@/components/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select'
import { Separator } from '@/components/separator'
import { usePluginMutations } from '@/entities/plugin/api/usePluginMutations.ts'
import { usePlugins } from '@/entities/plugin/api/usePlugins.ts'
import type { PluginStatusInfo } from '@/entities/plugin/model/types.ts'
import { Search } from '@/features/Search/components/Search'
import { ThemeSwitch } from '@/features/ThemeSwitch/ThemeSwitch'
import { ProfileDropdown } from '@/features/auth/ProfileDropdown'
import { Skeleton } from '@/shared/ui/skeleton.tsx'
import { ConfigDrawer } from '@/widgets/ConfigDrawer/ConfigDrawer'
import { Main } from '@/widgets/Layout/Main'
import { Header } from '@/widgets/Layout/components/header'

type PluginType = 'all' | 'active' | 'inactive'

const pluginText = new Map<PluginType, string>([
  ['all', 'All Plugins'],
  ['active', 'Active'],
  ['inactive', 'Inactive'],
])

function PluginConnectButton({ plugin }: { plugin: PluginStatusInfo }) {
  const { enablePlugin, disablePlugin } = usePluginMutations()

  const isLoading = enablePlugin.isPending || disablePlugin.isPending

  if (plugin.status === 'active') {
    return (
      <Button
        variant="outline"
        size="sm"
        disabled={isLoading}
        onClick={() => disablePlugin.mutate(plugin.manifest.id)}
        className="border-red-300 bg-red-50 hover:bg-red-100 dark:border-red-700 dark:bg-red-950 dark:hover:bg-red-900"
      >
        {isLoading ? 'Disabling...' : 'Disable'}
      </Button>
    )
  }

  if (plugin.status === 'inactive') {
    return (
      <Button
        variant="outline"
        size="sm"
        disabled={isLoading}
        onClick={() => enablePlugin.mutate(plugin.manifest.id)}
        className="border-blue-300 bg-blue-50 hover:bg-blue-100 dark:border-blue-700 dark:bg-blue-950 dark:hover:bg-blue-900"
      >
        {isLoading ? 'Enabling...' : 'Enable'}
      </Button>
    )
  }

  // Handle 'failed' state
  return (
    <Button variant="destructive" size="sm" disabled>
      Failed
    </Button>
  )
}

export function PluginsPage() {
  const { t } = useTranslation('plugins')
  const [searchParams, setSearchParams] = useSearchParams()
  const { data, isLoading: isPluginsLoading } = usePlugins()

  // ... (state for filters: searchTerm, appType, sort) ...
  const [searchTerm, setSearchTerm] = useState(() => searchParams.get('filter') || '')
  const [appType, setAppType] = useState<PluginType>(
    () => (searchParams.get('type') as PluginType) || 'all',
  )
  const [sort, setSort] = useState<'asc' | 'desc'>(
    () => (searchParams.get('sort') as 'asc' | 'desc') || 'asc',
  )

  // 4. Update filtering logic to use the new data structure
  const filteredPlugins = useMemo(() => {
    if (!data?.statuses) return []
    return data.statuses
      .sort((a, b) =>
        sort === 'asc'
          ? a.manifest.name.localeCompare(b.manifest.name)
          : b.manifest.name.localeCompare(a.manifest.name),
      )
      .filter((p) =>
        appType === 'active'
          ? p.status === 'active'
          : appType === 'inactive'
            ? p.status === 'inactive'
            : true,
      )
      .filter((p) => p.manifest.name.toLowerCase().includes(searchTerm.toLowerCase()))
  }, [data?.statuses, sort, appType, searchTerm])

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
              placeholder={t('SEARCH_PLACEHOLDER')}
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
                <SelectItem value="active">Active</SelectItem>
                <SelectItem value="inactive">Inactive</SelectItem>
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
          {isPluginsLoading
            ? // Skeleton loading state
              Array.from({ length: 3 }).map((_, i) => (
                <li key={i} className="space-y-8 rounded-lg border p-4">
                  <div className="flex items-center justify-between">
                    <Skeleton className="h-10 w-10 rounded-lg" />
                    <Skeleton className="h-8 w-20 rounded-md" />
                  </div>
                  <div>
                    <Skeleton className="mb-2 h-5 w-1/2" />
                    <Skeleton className="h-4 w-full" />
                  </div>
                </li>
              ))
            : filteredPlugins.map((plugin) => (
                <li key={plugin.manifest.id} className="rounded-lg border p-4 hover:shadow-md">
                  <div className="mb-8 flex items-center justify-between">
                    <div className="bg-muted flex size-10 items-center justify-center rounded-lg p-2">
                      <Package />
                    </div>
                    {/* 5. Use the new dynamic button */}
                    <PluginConnectButton plugin={plugin} />
                  </div>
                  <div>
                    <Link to={plugin.manifest.frontend.route}>
                      <h2 className="mb-1 font-semibold hover:underline">{plugin.manifest.name}</h2>
                    </Link>
                    <p className="line-clamp-2 text-gray-500">{plugin.manifest.version}</p>
                  </div>
                </li>
              ))}
        </ul>
      </Main>
    </>
  )
}
