import { Separator } from '@/shared/ui/separator.tsx'
import { Skeleton } from '@/shared/ui/skeleton.tsx'
import { usePlugins } from '@/features/plugin/api/usePlugins.ts'

import { PluginCard } from './PluginCard.tsx'
import { PluginToolbar } from './PluginToolbar.tsx'
import { usePluginFilters } from '@/features/plugin/hooks/usePluginFilters.ts'

export function PluginList() {
  const { data, isLoading } = usePlugins()

  const {
    searchTerm,
    appType,
    sort,
    filteredPlugins,
    handleSearch,
    handleTypeChange,
    handleSortChange,
  } = usePluginFilters(data?.statuses)

  return (
    <div className="space-y-4">
      <PluginToolbar
        searchTerm={searchTerm}
        onSearchChange={handleSearch}
        appType={appType}
        onTypeChange={handleTypeChange}
        sort={sort}
        onSortChange={handleSortChange}
      />

      <Separator className="shadow-sm" />

      <ul className="grid gap-4 pt-4 pb-16 md:grid-cols-2 lg:grid-cols-3">
        {isLoading
          ? Array.from({ length: 3 }).map((_, i) => (
              <li key={i} className="space-y-8 rounded-lg border p-4">
                <div className="flex items-center justify-between">
                  <Skeleton className="h-10 w-10 rounded-lg" />
                  <Skeleton className="h-8 w-20 rounded-md" />
                </div>
                <div className="space-y-2">
                  <Skeleton className="h-5 w-1/2" />
                  <Skeleton className="h-4 w-full" />
                </div>
              </li>
            ))
          : filteredPlugins.map((plugin) => (
              <PluginCard key={plugin.manifest.id} plugin={plugin} />
            ))}
      </ul>
    </div>
  )
}
