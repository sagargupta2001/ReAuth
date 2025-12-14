import type { DragEvent, ElementType } from 'react'

// Import missing icons (Play for Start)
import { Box, CheckCircle, Loader2, Lock, Play, ShieldAlert, Split, XCircle } from 'lucide-react'

import { type NodeMetadata, useNodes } from '@/features/flow-builder/api/useNodes'
import { cn } from '@/lib/utils'

const IconMap: Record<string, ElementType> = {
  Lock: Lock,
  Split: Split,
  ShieldAlert: ShieldAlert,
  CheckCircle: CheckCircle,
  XCircle: XCircle,
  Play: Play, // Added Play icon mapping
  Box: Box,
}

export function NodePalette() {
  const { data: nodes, isLoading } = useNodes()

  const onDragStart = (event: DragEvent, node: NodeMetadata) => {
    // 1. Pass Identification
    event.dataTransfer.setData('application/reactflow/type', node.id)
    event.dataTransfer.setData('application/reactflow/category', node.category)

    // 2. [CRITICAL FIX] Pass Outputs
    // This allows the Node Component to render the correct handles instantly on drop
    if (node.outputs) {
      event.dataTransfer.setData('application/reactflow/outputs', JSON.stringify(node.outputs))
    }

    event.dataTransfer.effectAllowed = 'move'
  }

  if (isLoading) {
    return (
      <aside className="bg-muted/10 text-muted-foreground flex w-64 flex-col items-center justify-center border-r">
        <Loader2 className="h-5 w-5 animate-spin" />
      </aside>
    )
  }

  return (
    <aside className="bg-muted/10 flex w-64 flex-col border-r">
      <div className="text-muted-foreground border-b p-4 text-xs font-semibold uppercase">
        Components
      </div>
      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        {(nodes || []).map((node) => {
          const IconComponent = IconMap[node.icon] || Box

          return (
            <div
              key={node.id}
              className={cn(
                'bg-card hover:border-primary/50 flex cursor-grab items-center gap-3 rounded-md border p-3 shadow-sm transition-colors active:cursor-grabbing',
              )}
              draggable
              onDragStart={(e) => onDragStart(e, node)}
            >
              <IconComponent className="text-muted-foreground h-4 w-4" />

              <div className="flex flex-col">
                <span className="text-sm leading-none font-medium">{node.display_name}</span>
                <span className="text-muted-foreground mt-1 line-clamp-1 text-[10px]">
                  {node.description}
                </span>
              </div>
            </div>
          )
        })}
      </div>
    </aside>
  )
}
