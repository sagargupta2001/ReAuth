import type { DragEvent, ElementType } from 'react'

import { Box, CheckCircle, Loader2, Lock, ShieldAlert, Split, XCircle } from 'lucide-react'

import { type NodeMetadata, useNodes } from '@/features/flow-builder/api/useNodes'
// Ensure type is imported
import { cn } from '@/lib/utils'

// 1. Icon Mapping: Convert string names from API to React Components
const IconMap: Record<string, ElementType> = {
  Lock: Lock,
  Split: Split,
  ShieldAlert: ShieldAlert,
  CheckCircle: CheckCircle,
  XCircle: XCircle,
  // Default fallback
  Box: Box,
}

export function NodePalette() {
  const { data: nodes, isLoading } = useNodes()

  const onDragStart = (event: DragEvent, node: NodeMetadata) => {
    // 2. Use correct property: node.id instead of nodeType
    // Pass both Type and Category so the Canvas knows how to render it
    event.dataTransfer.setData('application/reactflow/type', node.id)
    event.dataTransfer.setData('application/reactflow/category', node.category)
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
        {/* 3. Safety Check: Use nodes?.map or (nodes || []).map */}
        {(nodes || []).map((node) => {
          // 4. Resolve Icon
          const IconComponent = IconMap[node.icon] || Box

          return (
            <div
              key={node.id} // Use unique ID from API
              className={cn(
                'bg-card hover:border-primary/50 flex cursor-grab items-center gap-3 rounded-md border p-3 shadow-sm transition-colors active:cursor-grabbing',
              )}
              draggable
              onDragStart={(e) => onDragStart(e, node)}
            >
              {/* 5. Render resolved component */}
              <IconComponent className="text-muted-foreground h-4 w-4" />

              <div className="flex flex-col">
                {/* 6. Use display_name instead of label */}
                <span className="text-sm leading-none font-medium">{node.display_name}</span>
                {/* Optional description */}
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
