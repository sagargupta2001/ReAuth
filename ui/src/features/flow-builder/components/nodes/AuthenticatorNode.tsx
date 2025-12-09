import { Handle, type NodeProps, Position } from '@xyflow/react'
import { LockKeyhole } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/card'
import { cn } from '@/lib/utils'

export function AuthenticatorNode({ data, selected }: NodeProps) {
  return (
    <div className="group relative">
      {/* Input Handle */}
      <Handle
        type="target"
        position={Position.Top}
        className="bg-muted-foreground border-background top-[-6px] h-3 w-3 border-2"
      />

      <Card
        className={cn(
          'bg-card w-64 border-2 shadow-sm transition-all',
          selected
            ? 'border-primary ring-primary/20 shadow-md ring-2'
            : 'border-border hover:border-primary/50',
        )}
      >
        <CardHeader className="bg-muted/30 flex flex-row items-center gap-3 space-y-0 border-b p-3 pb-2">
          <div className="bg-background rounded-md border p-1.5 shadow-sm">
            <LockKeyhole className="text-primary h-4 w-4" />
          </div>
          <div className="flex flex-col">
            <span className="text-muted-foreground text-[10px] font-bold tracking-wider uppercase">
              Authenticator
            </span>
            <CardTitle
              className="w-40 truncate text-sm leading-none font-medium"
              title={data.label as string}
            >
              {data.label as string}
            </CardTitle>
          </div>
        </CardHeader>

        <CardContent className="text-muted-foreground p-3 pt-2 text-xs">
          {/* We can show brief config summary here later */}
          Input Required
        </CardContent>
      </Card>

      {/* Output Handles */}
      <div className="absolute right-0 -bottom-3 left-0 flex justify-around px-4">
        {/* Success Handle */}
        <div className="group/handle relative">
          <Handle
            type="source"
            position={Position.Bottom}
            id="success"
            className="border-background !static h-3 w-3 border-2 bg-green-500"
          />
          <div className="bg-popover text-popover-foreground pointer-events-none absolute top-4 left-1/2 z-50 -translate-x-1/2 rounded border px-1.5 py-0.5 text-[10px] whitespace-nowrap opacity-0 shadow-sm transition-opacity group-hover/handle:opacity-100">
            Success
          </div>
        </div>

        {/* Failure Handle */}
        <div className="group/handle relative">
          <Handle
            type="source"
            position={Position.Bottom}
            id="failure"
            className="bg-destructive border-background !static h-3 w-3 border-2"
          />
          <div className="bg-popover text-popover-foreground pointer-events-none absolute top-4 left-1/2 z-50 -translate-x-1/2 rounded border px-1.5 py-0.5 text-[10px] whitespace-nowrap opacity-0 shadow-sm transition-opacity group-hover/handle:opacity-100">
            Failure
          </div>
        </div>
      </div>
    </div>
  )
}
