import { useEffect, useMemo } from 'react'

import { type Edge, type Node, ReactFlowProvider } from '@xyflow/react'
import { Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'
import { useFlowDraft } from '@/features/flow-builder/api/useFlowDraft'
import { BuilderHeader } from '@/features/flow-builder/components/BuilderHeader'
import { FlowCanvas } from '@/features/flow-builder/components/FlowCanvas'
import { NodeInspector } from '@/features/flow-builder/components/NodeInspector'
import { NodePalette } from '@/features/flow-builder/components/NodePalette'
import { useFlowBuilderStore } from '@/features/flow-builder/store/flowBuilderStore'
import { HarborResourceActions } from '@/features/harbor/components/HarborResourceActions'
import { useActiveTheme } from '@/features/theme/api/useActiveTheme'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { Button } from '@/components/button'

export function FlowBuilderPage() {
  const { flowId } = useParams()
  const realm = useActiveRealm()
  // Ensure we have a string, though the router guarantees this param exists
  const draftId = flowId!

  const { data: draft, isLoading, isError } = useFlowDraft(flowId!)
  const { setGraph, reset, nodes, nodeTypes, publishError, selectNode } = useFlowBuilderStore()
  const { data: activeTheme } = useActiveTheme()

  const publishErrorNodeIds = useMemo(() => {
    if (!publishError) return []
    const matches = Array.from(publishError.matchAll(/node_id=([A-Za-z0-9_-]+)/g))
    const ids = matches.map((match) => match[1]).filter(Boolean)
    return Array.from(new Set(ids))
  }, [publishError])

  const missingTemplates = useMemo(() => {
    if (!activeTheme) return []
    const pages = activeTheme.pages
    const pageKeys = new Set(pages.map((page) => page.key))
    const nodeTypeMap = new Map(nodeTypes.map((node) => [node.id, node]))

    const keys = new Set<string>()
    nodes.forEach((node) => {
      const nodeType = node.type ?? ''
      const nodeDefinition = nodeTypeMap.get(nodeType)
      if (!nodeDefinition?.supports_ui) {
        return
      }
      const config = (node.data as { config?: Record<string, unknown> })?.config
      const ui =
        typeof config?.ui === 'object' && config.ui
          ? (config.ui as Record<string, unknown>)
          : {}
      const explicit =
        typeof ui?.page_key === 'string'
          ? (ui.page_key as string)
          : typeof config?.template_key === 'string'
            ? (config.template_key as string)
            : undefined
      const fallback = nodeDefinition?.default_template_key ?? undefined
      const key = explicit || fallback
      if (key) {
        keys.add(key)
      }
    })

    const missing = Array.from(keys).filter((key) => !pageKeys.has(key))
    return missing
  }, [activeTheme, nodeTypes, nodes])

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
          actions={
            flowId && realm ? (
              <HarborResourceActions
                scope="flow"
                id={flowId}
                resourceLabel={draft?.name || 'Flow'}
                invalidateKeys={[
                  ['flows', realm],
                  ['flow-draft', realm, flowId],
                  ['flow-drafts', realm],
                  ['flow-versions', flowId],
                ]}
              />
            ) : null
          }
        />

        {publishError && (
          <div className="border-b px-6 py-3">
            <Alert variant="destructive">
              <AlertTitle>Publish blocked</AlertTitle>
              <AlertDescription className="flex flex-wrap items-center gap-3">
                <span>{publishError}</span>
                {publishErrorNodeIds.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {publishErrorNodeIds.map((nodeId) => (
                      <Button
                        key={nodeId}
                        variant="outline"
                        size="sm"
                        onClick={() => selectNode(nodeId)}
                      >
                        Open {nodeId}
                      </Button>
                    ))}
                  </div>
                )}
              </AlertDescription>
            </Alert>
          </div>
        )}

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
