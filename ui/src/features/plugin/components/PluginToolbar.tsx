import type { ChangeEvent } from 'react'

import { ArrowDownAZ, ArrowUpAZ, SlidersHorizontal } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Input } from '@/shared/ui/input.tsx'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/shared/ui/select.tsx'

import type { PluginType, SortType } from '@/features/plugin/hooks/usePluginFilters.ts'

interface Props {
  searchTerm: string
  onSearchChange: (e: ChangeEvent<HTMLInputElement>) => void
  appType: PluginType
  onTypeChange: (val: PluginType) => void
  sort: SortType
  onSortChange: (val: SortType) => void
}

export function PluginToolbar({
  searchTerm,
  onSearchChange,
  appType,
  onTypeChange,
  sort,
  onSortChange,
}: Props) {
  const { t } = useTranslation('plugins')

  return (
    <div className="my-4 flex items-end justify-between sm:my-0 sm:items-center">
      <div className="flex flex-col gap-4 sm:my-4 sm:flex-row">
        <Input
          placeholder={t('SEARCH_PLACEHOLDER')}
          className="h-9 w-40 lg:w-[250px]"
          value={searchTerm}
          onChange={onSearchChange}
        />
        <Select value={appType} onValueChange={onTypeChange}>
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('FILTER_OPTIONS.ALL_PLUGINS')}</SelectItem>
            <SelectItem value="active">{t('FILTER_OPTIONS.ACTIVE')}</SelectItem>
            <SelectItem value="inactive">{t('FILTER_OPTIONS.INACTIVE')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <Select value={sort} onValueChange={onSortChange}>
        <SelectTrigger className="w-16">
          <SelectValue>
            <SlidersHorizontal size={18} />
          </SelectValue>
        </SelectTrigger>
        <SelectContent align="end">
          <SelectItem value="asc">
            <div className="flex items-center gap-4">
              <ArrowUpAZ size={16} />
              <span>{t('SORT_OPTIONS.ASCENDING')}</span>
            </div>
          </SelectItem>
          <SelectItem value="desc">
            <div className="flex items-center gap-4">
              <ArrowDownAZ size={16} />
              <span>{t('SORT_OPTIONS.DESCENDING')}</span>
            </div>
          </SelectItem>
        </SelectContent>
      </Select>
    </div>
  )
}
