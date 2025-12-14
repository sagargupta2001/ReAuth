import { Handle, Position } from '@xyflow/react'
import { Play } from 'lucide-react'

import { cn } from '@/lib/utils'

export function StartNode({ selected }: { selected?: boolean }) {
  return (
    <div
      className={cn(
        'bg-card relative min-w-[150px] rounded-md border-2 border-green-500 p-3 shadow-md transition-all',
        selected && 'ring-2 ring-green-400 ring-offset-2',
      )}
    >
      <div className="flex items-center gap-3">
        <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-100">
          <Play className="h-4 w-4 text-green-600" fill="currentColor" />
        </div>
        <div className="flex flex-col">
          <span className="text-sm font-semibold">Start</span>
          <span className="text-muted-foreground text-[10px]">Flow Entry</span>
        </div>
      </div>

      {/* Start Node only has an Output handle */}
      <Handle
        type="source"
        position={Position.Bottom}
        id="default"
        className="!h-3 !w-3 !bg-green-500"
      />
    </div>
  )
}
