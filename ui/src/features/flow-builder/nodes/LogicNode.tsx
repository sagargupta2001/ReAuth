import { Handle, type NodeProps, Position } from '@xyflow/react'
import { Split } from 'lucide-react'

import { cn } from '@/lib/utils.ts'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card.tsx'

export function LogicNode({ data, selected }: NodeProps) {
  const outputs = Array.isArray(data.outputs) ? (data.outputs as string[]) : ['true', 'false']

  const getHandleColor = (id: string) => {
    switch (id) {
      case 'true':
        return 'bg-green-500'
      case 'false':
        return 'bg-destructive'
      default:
        return 'bg-primary'
    }
  }

  return (
    <div className="group relative">
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
            <Split className="text-primary h-4 w-4" />
          </div>
          <div className="flex flex-col">
            <span className="text-muted-foreground text-[10px] font-bold tracking-wider uppercase">
              Logic
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
          Decision
        </CardContent>
      </Card>

      <div className="absolute right-0 -bottom-3 left-0 flex justify-around px-4">
        {outputs.map((outputId) => (
          <div key={outputId} className="group/handle relative">
            <Handle
              type="source"
              position={Position.Bottom}
              id={outputId}
              className={cn('border-background !static h-3 w-3 border-2', getHandleColor(outputId))}
            />
            <div className="bg-popover text-popover-foreground pointer-events-none absolute top-4 left-1/2 z-50 -translate-x-1/2 rounded border px-1.5 py-0.5 text-[10px] whitespace-nowrap capitalize opacity-0 shadow-sm transition-opacity group-hover/handle:opacity-100">
              {outputId}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
