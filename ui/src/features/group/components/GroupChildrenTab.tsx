import { useMemo, useState } from 'react'

import {
  type ColumnDef,
  type OnChangeFn,
  type PaginationState,
  type SortingState,
} from '@tanstack/react-table'
import { Folder } from 'lucide-react'

import { Badge } from '@/components/badge'
import { useRealmNavigate } from '@/entities/realm/lib/navigation'
import { useGroupChildrenList } from '@/features/group/api/useGroupChildren'
import type { GroupTreeNode } from '@/features/group-tree/model/types'
import { DataTableColumnHeader } from '@/shared/ui/data-table'
import { DataTable } from '@/shared/ui/data-table/data-table'
import { DataTableSkeleton } from '@/shared/ui/data-table/data-table-skeleton'

interface GroupChildrenTabProps {
  groupId: string
}

export function GroupChildrenTab({ groupId }: GroupChildrenTabProps) {
  const navigate = useRealmNavigate()
  const [searchTerm, setSearchTerm] = useState('')
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  })
  const [sorting, setSorting] = useState<SortingState>([{ id: 'name', desc: false }])

  const { data, isLoading } = useGroupChildrenList(groupId, {
    page: pagination.pageIndex + 1,
    per_page: pagination.pageSize,
    sort_by: sorting[0]?.id,
    sort_dir: sorting[0]?.desc ? 'desc' : 'asc',
    q: searchTerm,
  })

  const columns = useMemo<ColumnDef<GroupTreeNode>[]>(
    () => [
      {
        accessorKey: 'name',
        header: ({ column }) => <DataTableColumnHeader column={column} title="Group" />,
        cell: ({ row }) => (
          <div className="flex items-center gap-2 font-medium">
            <Folder className="text-muted-foreground h-4 w-4" />
            {row.getValue('name')}
          </div>
        ),
        enableSorting: true,
      },
      {
        accessorKey: 'description',
        header: 'Description',
        cell: ({ row }) => (
          <div className="text-muted-foreground max-w-[400px] truncate">
            {row.getValue('description') || '-'}
          </div>
        ),
      },
      {
        id: 'children',
        header: 'Children',
        cell: ({ row }) =>
          row.original.has_children ? (
            <Badge variant="secondary">Has children</Badge>
          ) : (
            <span className="text-muted-foreground text-xs">Leaf</span>
          ),
        size: 120,
      },
    ],
    [],
  )

  const handlePaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(pagination) : updater
    setPagination(nextState)
  }

  const handleSortingChange: OnChangeFn<SortingState> = (updater) => {
    const nextState = typeof updater === 'function' ? updater(sorting) : updater
    setSorting(nextState)
  }

  const handleSearch = (value: string) => {
    setSearchTerm(value)
    setPagination((prev) => ({ ...prev, pageIndex: 0 }))
  }

  if (isLoading) {
    return (
      <div className="h-[calc(100vh-440px)]">
        <DataTableSkeleton columnCount={3} rowCount={8} />
      </div>
    )
  }

  return (
    <DataTable
      columns={columns}
      data={data?.data || []}
      pageCount={data?.meta.total_pages || 0}
      pagination={pagination}
      onPaginationChange={handlePaginationChange}
      sorting={sorting}
      onSortingChange={handleSortingChange}
      searchKey="name"
      searchPlaceholder="Filter sub-groups..."
      searchValue={searchTerm}
      onSearch={handleSearch}
      onRowClick={(group) => navigate(`/groups/${group.id}/settings`)}
      className="h-[calc(100vh-482px)]"
    />
  )
}
