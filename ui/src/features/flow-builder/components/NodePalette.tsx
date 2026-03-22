import { type DragEvent, type ElementType, useEffect, useMemo, useState } from 'react'

// Import missing icons (Play for Start)
import {
  Box,
  CheckCircle,
  Loader2,
  Lock,
  Play,
  ShieldAlert,
  Split,
  UserPlus,
  XCircle,
} from 'lucide-react'

import { Input } from '@/components/input'
import { type NodeMetadata, useNodes } from '@/features/flow-builder/api/useNodes'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore'
import { cn } from '@/lib/utils'

const IconMap: Record<string, ElementType> = {
  Lock: Lock,
  Split: Split,
  ShieldAlert: ShieldAlert,
  CheckCircle: CheckCircle,
  XCircle: XCircle,
  Play: Play, // Added Play icon mapping
  UserPlus: UserPlus,
  Box: Box,
}

export function NodePalette() {
  const { data: nodes, isLoading } = useNodes()
  const setNodeTypes = useFlowBuilderStore((state) => state.setNodeTypes)
  const [searchTerm, setSearchTerm] = useState('')

  useEffect(() => {
    if (nodes) {
      setNodeTypes(nodes)
    }
  }, [nodes, setNodeTypes])

  const filteredNodes = useMemo(() => {
    const term = searchTerm.trim().toLowerCase()
    if (!term) return nodes || []
    return (nodes || []).filter((node) => {
      const haystack = `${node.display_name} ${node.description} ${node.id}`.toLowerCase()
      return haystack.includes(term)
    })
  }, [nodes, searchTerm])

  const onDragStart = (event: DragEvent, node: NodeMetadata) => {
    // 1. Pass Identification
    event.dataTransfer.setData('application/reactflow/type', node.id)
    event.dataTransfer.setData('application/reactflow/category', node.category)
    event.dataTransfer.setData('application/reactflow/label', node.display_name)
    event.dataTransfer.setData(
      'application/reactflow/default-template-key',
      node.default_template_key || '',
    )

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
      <div className="border-b p-4">
        <Input
          value={searchTerm}
          onChange={(event) => setSearchTerm(event.target.value)}
          placeholder="search components..."
          className="h-8 text-xs"
        />
      </div>
      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        {filteredNodes.map((node) => {
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
