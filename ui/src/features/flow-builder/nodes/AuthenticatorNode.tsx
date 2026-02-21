import { Handle, type NodeProps, Position } from '@xyflow/react'
import { LockKeyhole } from 'lucide-react'

import { cn } from '@/lib/utils.ts'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card.tsx'

export function AuthenticatorNode({ data, selected }: NodeProps) {
  // 1. Get outputs from backend data, or default to standard auth outputs
  const outputs = Array.isArray(data.outputs) ? (data.outputs as string[]) : ['success', 'failure']

  // Helper to determine color based on handle name
  const getHandleColor = (id: string) => {
    switch (id) {
      case 'success':
        return 'bg-green-500'
      case 'failure':
        return 'bg-destructive'
      case 'continue':
        return 'bg-blue-500' // Distinct color for SSO/Cookie flow
      default:
        return 'bg-primary'
    }
  }

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
              {/* Show the specific type if available (e.g. Cookie vs Password) */}
              {(data.config as Record<string, unknown>)?.auth_type
                ? String((data.config as Record<string, unknown>).auth_type)
                    .split('.')
                    .pop()
                : 'Authenticator'}
            </span>
            <CardTitle
              className="w-40 truncate text-sm leading-none font-medium"
              title={data.label as string}
            >
              {data.label as string}
            </CardTitle>
          </div>
        </CardHeader>

        <CardContent className="text-muted-foreground p-3 pt-2 text-xs">Input Required</CardContent>
      </Card>

      {/* Dynamic Output Handles */}
      <div className="absolute right-0 -bottom-3 left-0 flex justify-around px-4">
        {outputs.map((outputId) => (
          <div key={outputId} className="group/handle relative">
            <Handle
              type="source"
              position={Position.Bottom}
              id={outputId} // <--- This allows the backend to target specific paths
              className={cn('border-background !static h-3 w-3 border-2', getHandleColor(outputId))}
            />
            {/* Tooltip Label */}
            <div className="bg-popover text-popover-foreground pointer-events-none absolute top-4 left-1/2 z-50 -translate-x-1/2 rounded border px-1.5 py-0.5 text-[10px] whitespace-nowrap capitalize opacity-0 shadow-sm transition-opacity group-hover/handle:opacity-100">
              {outputId}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
