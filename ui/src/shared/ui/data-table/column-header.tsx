import { ArrowDownIcon, ArrowUpIcon, CaretSortIcon, EyeNoneIcon } from '@radix-ui/react-icons'
import { type Column } from '@tanstack/react-table'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/dropdown-menu'
import { cn } from '@/lib/utils'

type DataTableColumnHeaderProps<TData, TValue> = React.HTMLAttributes<HTMLDivElement> & {
  column: Column<TData, TValue>
  title: string
}

export function DataTableColumnHeader<TData, TValue>({
  column,
  title,
  className,
}: DataTableColumnHeaderProps<TData, TValue>) {
  if (!column.getCanSort())
    return (
      <div className={cn(className)}>
        <span className="font-bold text-amber-50">{title}</span>
      </div>
    )

  return (
    <div className={cn('flex items-center space-x-2 h-12', className)}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <span className="flex h-8 cursor-pointer items-center">
            <span className="font-bold text-amber-50">{title}</span>
            {column.getIsSorted() === 'desc' ? (
              <ArrowDownIcon className="ms-2 h-4 w-4" />
            ) : column.getIsSorted() === 'asc' ? (
              <ArrowUpIcon className="ms-2 h-4 w-4" />
            ) : (
              <CaretSortIcon className="ms-2 h-4 w-4" />
            )}
          </span>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start">
          <DropdownMenuItem onClick={() => column.toggleSorting(false)}>
            <ArrowUpIcon className="text-muted-foreground/70 size-3.5" />
            Asc
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => column.toggleSorting(true)}>
            <ArrowDownIcon className="text-muted-foreground/70 size-3.5" />
            Desc
          </DropdownMenuItem>
          {column.getCanHide() && (
            <>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => column.toggleVisibility(false)}>
                <EyeNoneIcon className="text-muted-foreground/70 size-3.5" />
                Hide
              </DropdownMenuItem>
            </>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
