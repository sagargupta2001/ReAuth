import { Skeleton } from '@/components/skeleton'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/table'

interface DataTableSkeletonProps {
  columnCount?: number
  rowCount?: number
}

export function DataTableSkeleton({ columnCount = 4, rowCount = 5 }: DataTableSkeletonProps) {
  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            {Array.from({ length: columnCount }).map((_, i) => (
              <TableHead key={i}>
                <Skeleton className="h-4 w-[100px]" />
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {Array.from({ length: rowCount }).map((_, i) => (
            <TableRow key={i}>
              {Array.from({ length: columnCount }).map((_, j) => (
                <TableCell key={j}>
                  <Skeleton className="h-4 w-[80px]" />
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
