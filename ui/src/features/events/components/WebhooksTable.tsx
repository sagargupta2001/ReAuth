import { useEffect, useMemo, useState } from 'react'
import type { OnChangeFn, PaginationState, SortingState } from '@tanstack/react-table'
import { useSearchParams } from 'react-router-dom'

import { useRealmNavigate } from '@/entities/realm/lib/navigation.logic'
import { useWebhooks } from '@/features/events/api/useWebhooks'
import { webhookColumns, type WebhookRow } from '@/features/events/components/WebhookColumns'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

export function WebhooksTable() {
  const navigate = useRealmNavigate()
  const [searchParams, setSearchParams] = useSearchParams()

  const webhookPage = Number(searchParams.get('webhook_page')) || 1
  const webhookPerPage = Number(searchParams.get('webhook_per_page')) || 10
  const webhookSortBy = searchParams.get('webhook_sort_by') || 'updated_at'
  const webhookSortDir = (searchParams.get('webhook_sort_dir') as 'asc' | 'desc') || 'desc'
  const webhookQuery = searchParams.get('webhook_q') || ''

  const [webhookSearch, setWebhookSearch] = useState(webhookQuery)

  const { data: webhookData, isLoading, isError } = useWebhooks({
    page: webhookPage,
    per_page: webhookPerPage,
    sort_by: webhookSortBy,
    sort_dir: webhookSortDir,
    q: webhookQuery || undefined,
  })

  useEffect(() => {
    setWebhookSearch(webhookQuery)
  }, [webhookQuery])

  useEffect(() => {
    const timer = setTimeout(() => {
      if (webhookSearch !== webhookQuery) {
        const params = new URLSearchParams(searchParams)
        if (webhookSearch) {
          params.set('webhook_q', webhookSearch)
        } else {
          params.delete('webhook_q')
        }
        params.set('webhook_page', '1')
        setSearchParams(params)
      }
    }, 400)
    return () => clearTimeout(timer)
  }, [searchParams, setSearchParams, webhookQuery, webhookSearch])

  const webhookRows = useMemo<WebhookRow[]>(() => {
    const rows = webhookData?.data ?? []
    return rows.map((details) => {
      const enabledSubscriptions = details.subscriptions.filter((sub) => sub.enabled)
      const subscriptionSummary = summarizeSubscriptions(
        enabledSubscriptions.map((sub) => sub.event_type),
      )
      const isFailing =
        details.endpoint.status !== 'active' || details.endpoint.consecutive_failures > 0

      return {
        id: details.endpoint.id,
        url: details.endpoint.url,
        http_method: details.endpoint.http_method || 'POST',
        status: isFailing ? 'failing' : 'active',
        subscriptions: subscriptionSummary,
        updated_at: details.endpoint.updated_at,
      }
    })
  }, [webhookData])

  const pagination = useMemo<PaginationState>(
    () => ({ pageIndex: webhookPage - 1, pageSize: webhookPerPage }),
    [webhookPage, webhookPerPage],
  )
  const sorting = useMemo<SortingState>(
    () => [{ id: webhookSortBy, desc: webhookSortDir === 'desc' }],
    [webhookSortBy, webhookSortDir],
  )

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const next = typeof updater === 'function' ? updater(pagination) : updater
    const params = new URLSearchParams(searchParams)
    params.set('webhook_page', String(next.pageIndex + 1))
    params.set('webhook_per_page', String(next.pageSize))
    setSearchParams(params)
  }

  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const next = typeof updater === 'function' ? updater(sorting) : updater
    const params = new URLSearchParams(searchParams)
    if (next.length) {
      params.set('webhook_sort_by', next[0].id)
      params.set('webhook_sort_dir', next[0].desc ? 'desc' : 'asc')
    } else {
      params.delete('webhook_sort_by')
      params.delete('webhook_sort_dir')
    }
    params.set('webhook_page', '1')
    setSearchParams(params)
  }

  if (isLoading) {
    return <DataTableSkeleton columnCount={5} rowCount={8} />
  }

  if (isError) {
    return (
      <div className="py-6 text-center text-sm text-muted-foreground">
        Failed to load webhook endpoints.
      </div>
    )
  }

  return (
    <DataTable<WebhookRow, unknown>
      columns={webhookColumns}
      data={webhookRows}
      pageCount={webhookData?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      searchPlaceholder="Search endpoints..."
      searchValue={webhookSearch}
      onSearch={setWebhookSearch}
      className="h-[520px]"
      onRowClick={(row) => navigate(`/events/webhooks/${row.id}`)}
    />
  )
}

function summarizeSubscriptions(subscriptions: string[]) {
  if (subscriptions.length === 0) return 'No events'
  if (subscriptions.length <= 2) return subscriptions.join(', ')
  return `${subscriptions[0]}, ${subscriptions[1]} + ${subscriptions.length - 2} more`
}
