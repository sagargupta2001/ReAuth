import { Handle, type NodeProps, Position } from '@xyflow/react'
import { CheckCircle, XCircle } from 'lucide-react'

import { cn } from '@/lib/utils'

// We accept full NodeProps to access `data.label` and `type`
export function TerminalNode({ data, selected, type }: NodeProps) {
  // Determine style based on node ID/Type logic or label content
  // Assuming 'core.terminal.allow' vs 'core.terminal.deny'
  const isAllow = type?.includes('allow') || data.label?.toString().toLowerCase().includes('allow')

  const borderColor = isAllow ? 'border-blue-500' : 'border-red-500'
  const iconColor = isAllow ? 'text-blue-600' : 'text-red-600'
  const bgColor = isAllow ? 'bg-blue-50' : 'bg-red-50'

  const Icon = isAllow ? CheckCircle : XCircle

  return (
    <div
      className={cn(
        'bg-card relative min-w-[150px] rounded-md border-2 p-3 shadow-md transition-all',
        borderColor,
        selected && 'ring-2 ring-offset-2',
        selected && (isAllow ? 'ring-blue-400' : 'ring-red-400'),
      )}
    >
      {/* Terminal Node only has an Input handle */}
      <Handle
        type="target"
        position={Position.Top}
        className={cn('!h-3 !w-3', isAllow ? '!bg-blue-500' : '!bg-red-500')}
      />

      <div className="flex items-center gap-3">
        <div className={cn('flex h-8 w-8 items-center justify-center rounded-full', bgColor)}>
          <Icon className={cn('h-4 w-4', iconColor)} />
        </div>
        <div className="flex flex-col">
          <span className="text-sm font-semibold">{data.label as string}</span>
          <span className="text-muted-foreground text-[10px]">End of Flow</span>
        </div>
      </div>
    </div>
  )
}
