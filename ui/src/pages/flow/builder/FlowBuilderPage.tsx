import { useEffect, useMemo } from 'react'

import { type Edge, type Node, ReactFlowProvider } from '@xyflow/react'
import { Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
import { BuilderHeader } from '@/features/flow-builder/components/BuilderHeader'
import { FlowCanvas } from '@/features/flow-builder/components/FlowCanvas'
import { NodeInspector } from '@/features/flow-builder/components/NodeInspector'
import { NodePalette } from '@/features/flow-builder/components/NodePalette'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore'
import { useActiveTheme } from '@/features/theme/api/useActiveTheme'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'

export function FlowBuilderPage() {
  const { flowId } = useParams()
  // Ensure we have a string, though the router guarantees this param exists
  const draftId = flowId!

  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)
  const { setGraph, reset, nodes } = useFlowBuilderStore()
  const { data: activeTheme } = useActiveTheme()

  const missingTemplates = useMemo(() => {
    if (!activeTheme) return []
    const pages = activeTheme.pages
    const pageKeys = new Set(pages.map((page) => page.key))
    const defaults: Record<string, string> = {
      'core.auth.password': 'login',
      'core.auth.otp': 'mfa',
    }

    const keys = new Set<string>()
    nodes.forEach((node) => {
      const config = (node.data as { config?: Record<string, unknown> })?.config
      const explicit = typeof config?.template_key === 'string' ? config.template_key : undefined
      const nodeType = node.type ?? ''
      const fallback = nodeType ? defaults[nodeType] || undefined : undefined
      const key = explicit || fallback
      if (key) {
        keys.add(key)
      }
    })

    const missing = Array.from(keys).filter((key) => !pageKeys.has(key))
    return missing
  }, [activeTheme, nodes])

  // Sync DB -> Store
  useEffect(() => {
    if (draft?.graph_json) {
      // React Flow expects { nodes: [], edges: [] }
      // If empty JSON {}, default to arrays
      const { nodes = [], edges = [] } = draft.graph_json as { nodes?: Node[]; edges?: Edge[] }
      setGraph(nodes, edges)
    }
  }, [draft, setGraph])

  // Cleanup on Unmount (Prevent old graph from flashing when opening a new one)
  useEffect(() => {
    return () => {
      reset()
    }
  }, [reset])

  if (isLoading)
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Flow Draft...</p>
      </div>
    )

  if (isError)
    return (
      <div className="text-destructive flex h-full w-full items-center justify-center">
        Failed to load flow. Please try again.
      </div>
    )

  return (
    <ReactFlowProvider>
      <div className="flex h-full w-full flex-col">
        <BuilderHeader
          flowName={draft?.name || 'Untitled Flow'}
          flowId={draftId}
          activeVersion={draft?.active_version}
        />

        {missingTemplates.length > 0 && (
          <div className="border-b px-6 py-3">
            <Alert variant="destructive">
              <AlertTitle>Missing Fluid templates</AlertTitle>
              <AlertDescription>
                The active theme does not contain the following templates used by this flow:{' '}
                {missingTemplates.join(', ')}. Users will fall back to system pages until you add
                them.
              </AlertDescription>
            </Alert>
          </div>
        )}

        <div className="relative flex flex-1 overflow-hidden">
          <NodePalette />

          <div className="relative h-full flex-1">
            <FlowCanvas />
          </div>

          <NodeInspector />
        </div>
      </div>
    </ReactFlowProvider>
  )
}
