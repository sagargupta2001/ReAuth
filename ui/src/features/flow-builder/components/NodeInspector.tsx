import { Separator } from '@radix-ui/react-select'
import { X } from 'lucide-react'

import { Button } from '@/shared/ui/button.tsx'

import { useFlowBuilderStore } from '../store/flowBuilderStore'

export function NodeInspector() {
  const selectedNodeId = useFlowBuilderStore((s) => s.selectedNodeId)
  const nodes = useFlowBuilderStore((s) => s.nodes)
  const selectNode = useFlowBuilderStore((s) => s.selectNode)

  const selectedNode = nodes.find((n) => n.id === selectedNodeId)

  if (!selectedNode) return null

  return (
    <aside className="bg-background flex h-full w-80 shrink-0 flex-col border-l transition-all duration-300 ease-in-out">
      {/* Header */}
      <div className="flex h-14 shrink-0 items-center justify-between border-b p-4">
        <div>
          <h3 className="text-sm font-semibold">Configuration</h3>
          <p className="text-muted-foreground mt-0.5 max-w-[180px] truncate font-mono text-xs">
            {selectedNode.data.label as string}
          </p>
        </div>
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => selectNode(null)}>
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* Content */}
      <div className="flex-1 space-y-6 overflow-y-auto p-4">
        {/* Common Settings */}
        <div className="space-y-3">
          <h4 className="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
            General
          </h4>
          <div className="space-y-1.5">
            <label className="text-xs font-medium">Label</label>
            <input
              className="border-input focus-visible:ring-ring flex h-8 w-full rounded-md border bg-transparent px-3 py-1 text-xs shadow-sm transition-colors focus-visible:ring-1 focus-visible:outline-none"
              value={selectedNode.data.label as string}
              // We will add onChange handler later to update store
              readOnly
            />
          </div>
          <div className="space-y-1.5">
            <label className="text-xs font-medium">Node ID</label>
            <div className="bg-muted text-muted-foreground rounded-md px-3 py-1.5 font-mono text-xs break-all">
              {selectedNode.id}
            </div>
          </div>
        </div>

        <Separator />

        {/* Dynamic Settings based on Node Type */}
        <div className="space-y-3">
          <h4 className="text-muted-foreground text-xs font-semibold tracking-wider uppercase">
            Parameters
          </h4>
          <p className="text-muted-foreground text-xs italic">
            {/* Placeholder until we integrate JSON Schema Forms */}
            No configuration options available for this node type yet.
          </p>
        </div>
      </div>
    </aside>
  )
}
