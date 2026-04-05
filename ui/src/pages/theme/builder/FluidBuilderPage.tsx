import { useEffect, useMemo, useRef, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useParams, useSearchParams } from 'react-router-dom'
import { toast } from 'sonner'

import type {
  ThemeNode,
  ThemeBlueprint,
  ThemeDraft,
  ThemePageTemplate,
} from '@/entities/theme/model/types'
import {
  extractNodesFromBlueprint,
  findNodeById,
  removeNodeById,
  updateNodeById,
  updateBlueprintWithNodes,
} from '@/features/fluid/lib/nodeUtils'
import { useTheme } from '@/features/theme/api/useTheme'
import { useThemePages } from '@/features/theme/api/useThemePages'
import { usePublishTheme } from '@/features/theme/api/usePublishTheme'
import { useSaveThemeDraft } from '@/features/theme/api/useSaveThemeDraft'
import { useThemeAssets } from '@/features/theme/api/useThemeAssets'
import { useThemeDraft } from '@/features/theme/api/useThemeDraft'
import { useUploadThemeAsset } from '@/features/theme/api/useUploadThemeAsset'
import { useThemeTemplateGaps } from '@/features/theme/api/useThemeTemplateGaps'
import { FluidBlocksPanel } from '@/features/fluid/components/FluidBlocksPanel'
import { FluidBuilderHeader } from '@/features/fluid/components/FluidBuilderHeader'
import { FluidCanvas } from '@/features/fluid/components/FluidCanvas'
import { FluidFloatingActionBar } from '@/features/fluid/components/FluidFloatingActionBar'
import { FluidInspector } from '@/features/fluid/components/FluidInspector'
import { FluidPrimarySidebar } from '@/features/fluid/components/FluidPrimarySidebar'
import { FluidThemeSettingsPanel } from '@/features/fluid/components/FluidThemeSettingsPanel'
import { HarborResourceActions } from '@/features/harbor/components/HarborResourceActions'
import {
  type ThemeValidationError,
  validateThemeDraft,
} from '@/features/fluid/lib/themeValidation'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { useActiveRealm } from '@/entities/realm/model/useActiveRealm'

const fallbackDraft: ThemeDraft = {
  tokens: {
    colors: {
      primary: 'var(--primary)',
      background: 'var(--background)',
      text: 'var(--foreground)',
      surface: 'var(--card)',
    },
    appearance: {
      mode: 'auto',
    },
    typography: {
      font_family: 'system-ui',
      base_size: 16,
    },
    radius: {
      base: 12,
    },
  },
  layout: {
    shell: 'CenteredCard',
    slots: ['main'],
  },
  nodes: [
    {
      node_key: 'login',
      blueprint: [],
    },
  ],
}

type DraftHistory = {
  past: ThemeDraft[]
  present: ThemeDraft
  future: ThemeDraft[]
}

const MAX_HISTORY = 50

const CUSTOM_PAGE_PREFIX = 'custom.'

function humanizePageKey(key: string) {
  const trimmed = key.startsWith(CUSTOM_PAGE_PREFIX)
    ? key.slice(CUSTOM_PAGE_PREFIX.length)
    : key
  const parts = trimmed.split(/[-_]/).filter(Boolean)
  if (parts.length === 0) return 'Custom Page'
  return parts
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

function slugifyPageKey(label: string) {
  return label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function collectNodeIds(nodes: ThemeNode[]) {
  const ids = new Set<string>()
  const visit = (node: ThemeNode) => {
    if (node.id) {
      ids.add(node.id)
    }
    node.children?.forEach(visit)
    if (node.slots) {
      Object.values(node.slots).forEach(visit)
    }
  }
  nodes.forEach(visit)
  return ids
}

function collectInputNames(nodes: ThemeNode[]) {
  const names = new Set<string>()
  const visit = (node: ThemeNode) => {
    const type = String(node.type || '')
    const component = String(node.component || '').toLowerCase()
    if (type === 'Input' || (type === 'Component' && component === 'input')) {
      const name = String(node.props?.name || '').trim()
      if (name) {
        names.add(name)
      }
    }
    if (node.children && node.children.length > 0) {
      node.children.forEach(visit)
    }
    if (node.slots) {
      Object.values(node.slots).forEach(visit)
    }
  }
  nodes.forEach(visit)
  return Array.from(names).sort()
}

export function FluidBuilderPage() {
  const { themeId } = useParams()
  const realmName = useActiveRealm()
  const [searchParams] = useSearchParams()
  const requestedPage = searchParams.get('page')?.trim() || null
  const { data, isLoading, isError } = useTheme(themeId)
  const appliedPageParam = useRef(false)
  const { data: pages = [] } = useThemePages(themeId)
  const {
    data: draft,
    isLoading: isDraftLoading,
    isError: isDraftError,
  } = useThemeDraft(themeId)
  const { data: templateGaps } = useThemeTemplateGaps(themeId)

  const [history, setHistory] = useState<DraftHistory>({
    past: [],
    present: fallbackDraft,
    future: [],
  })
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [activePageKey, setActivePageKey] = useState('login')
  const [activePanel, setActivePanel] = useState<'sections' | 'settings'>('sections')
  const [isInspecting, setIsInspecting] = useState(false)

  const activeDraft = useMemo(() => draft ?? fallbackDraft, [draft])
  const draftState = history.present
  const isActiveTheme = Boolean(data?.active_version_id)
  const missingTemplates = isActiveTheme ? templateGaps?.missing ?? [] : []

  const availablePages = useMemo<ThemePageTemplate[]>(() => {
    const known = new Map(pages.map((page) => [page.key, page]))
    for (const node of draftState.nodes) {
      if (known.has(node.node_key)) {
        continue
      }
      const label = humanizePageKey(node.node_key)
      known.set(node.node_key, {
        key: node.node_key,
        label,
        description: 'Custom page',
        category: 'custom',
        blueprint: node.blueprint,
      })
    }
    return Array.from(known.values())
  }, [draftState.nodes, pages])

  const commitDraft = (updater: (current: ThemeDraft) => ThemeDraft) => {
    setHistory((prev) => {
      const next = updater(prev.present)
      if (next === prev.present) {
        return prev
      }
      const trimmedPast =
        prev.past.length >= MAX_HISTORY ? prev.past.slice(1) : prev.past
      return {
        past: [...trimmedPast, prev.present],
        present: next,
        future: [],
      }
    })
  }

  const replaceDraft = (next: ThemeDraft) => {
    setHistory({
      past: [],
      present: next,
      future: [],
    })
  }

  const handleUndo = () => {
    setHistory((prev) => {
      if (prev.past.length === 0) return prev
      const previous = prev.past[prev.past.length - 1]
      return {
        past: prev.past.slice(0, -1),
        present: previous,
        future: [prev.present, ...prev.future],
      }
    })
  }

  const handleRedo = () => {
    setHistory((prev) => {
      if (prev.future.length === 0) return prev
      const next = prev.future[0]
      return {
        past: [...prev.past, prev.present],
        present: next,
        future: prev.future.slice(1),
      }
    })
  }

  useEffect(() => {
    if (activeDraft) {
      replaceDraft(activeDraft)
      setSelectedNodeId(null)
    }
  }, [activeDraft])

  useEffect(() => {
    if (availablePages.length === 0) return
    const isValid = availablePages.some((page) => page.key === activePageKey)
    if (!isValid) {
      setActivePageKey(availablePages[0].key)
    }
  }, [activePageKey, availablePages])

  useEffect(() => {
    if (!requestedPage || appliedPageParam.current) return
    const exists = availablePages.some((page) => page.key === requestedPage)
    if (exists) {
      setActivePageKey(requestedPage)
      appliedPageParam.current = true
    }
  }, [availablePages, requestedPage])

  const { mutateAsync: saveDraft, isPending: isSaving } = useSaveThemeDraft(themeId || '')
  const { mutateAsync: publishTheme, isPending: isPublishing } = usePublishTheme(themeId || '')
  const { data: assets = [] } = useThemeAssets(themeId)
  const { mutateAsync: uploadAsset, isPending: isUploading } = useUploadThemeAsset(themeId || '')
  const validationErrors = useMemo(
    () => validateThemeDraft(draftState),
    [draftState],
  )

  const handleSave = async () => {
    if (validationErrors.length > 0) {
      toast.error(
        `Theme draft has ${validationErrors.length} validation issue(s). First: ${validationErrors[0].message}`,
      )
      return
    }
    await saveDraft(history.present)
  }

  const handlePublish = async () => {
    if (validationErrors.length > 0) {
      toast.error(
        `Theme draft has ${validationErrors.length} validation issue(s). First: ${validationErrors[0].message}`,
      )
      return
    }
    await saveDraft(history.present)
    await publishTheme()
  }

  const handleResetPage = () => {
    if (!activePageKey) return
    setSelectedNodeId(null)

    if (data?.theme.is_system) {
      if (!activePageTemplate) return
      commitDraft((prev) => {
        const index = prev.nodes.findIndex((node) => node.node_key === activePageKey)
        if (index >= 0) {
          const nodes = [...prev.nodes]
          nodes[index] = {
            ...nodes[index],
            node_key: activePageKey,
            blueprint: activePageTemplate.blueprint,
          }
          return {
            ...prev,
            nodes,
          }
        }
        return {
          ...prev,
          nodes: [
            ...prev.nodes,
            {
              node_key: activePageKey,
              blueprint: activePageTemplate.blueprint,
            },
          ],
        }
      })
      return
    }

    commitDraft((prev) => ({
      ...prev,
      nodes: prev.nodes.filter((node) => node.node_key !== activePageKey),
    }))
  }

  const handleCreatePage = (label: string) => {
    const slug = slugifyPageKey(label)
    if (!slug) return
    const key = `${CUSTOM_PAGE_PREFIX}${slug}`
    const exists = availablePages.some((page) => page.key === key)
    if (exists) {
      setActivePageKey(key)
      return
    }
    const blueprint: ThemeBlueprint = {
      layout: 'default',
      nodes: [
        {
          id: `title-${Date.now()}`,
          type: 'Text',
          size: { width: 'fill', height: 'hug' },
          props: { text: label },
        },
      ],
    }
    commitDraft((prev) => ({
      ...prev,
      nodes: [
        ...prev.nodes,
        {
          node_key: key,
          blueprint,
        },
      ],
    }))
    setActivePageKey(key)
    setSelectedNodeId(null)
  }

  const activePageTemplate = useMemo<ThemePageTemplate | undefined>(
    () => availablePages.find((page) => page.key === activePageKey),
    [activePageKey, availablePages],
  )

  const activeNode = useMemo(
    () => draftState.nodes.find((node) => node.node_key === activePageKey),
    [draftState.nodes, activePageKey],
  )

  const activeBlueprint = useMemo(() => {
    if (activeNode) return activeNode.blueprint
    if (activePageTemplate) return activePageTemplate.blueprint
    return undefined
  }, [activeNode, activePageTemplate])

  const activeNodes = useMemo(() => {
    return extractNodesFromBlueprint(activeBlueprint).nodes
  }, [activeBlueprint])
  const inputNames = useMemo(() => collectInputNames(activeNodes), [activeNodes])
  const activeValidationErrors = useMemo<ThemeValidationError[]>(() => {
    if (validationErrors.length === 0) return []
    const activeIds = collectNodeIds(activeNodes)
    return validationErrors.filter(
      (error) => error.nodeId && activeIds.has(error.nodeId),
    )
  }, [activeNodes, validationErrors])

  useEffect(() => {
    if (selectedNodeId && !findNodeById(activeNodes, selectedNodeId)) {
      setSelectedNodeId(null)
    }
  }, [activeNodes, selectedNodeId])

  useEffect(() => {
    setSelectedNodeId(null)
  }, [activePageKey])

  const setNodes = (nextNodes: ThemeNode[]) => {
    commitDraft((prev) => {
      const index = prev.nodes.findIndex((node) => node.node_key === activePageKey)
      const hasNode = index >= 0
      const baseBlueprint = hasNode ? prev.nodes[index].blueprint : activeBlueprint
      const updatedNode = {
        node_key: activePageKey,
        blueprint: updateBlueprintWithNodes(baseBlueprint, nextNodes),
      }

      if (!hasNode) {
        return {
          ...prev,
          nodes: [...prev.nodes, updatedNode],
        }
      }

      const updatedNodes = [...prev.nodes]
      updatedNodes[index] = {
        ...updatedNodes[index],
        ...updatedNode,
      }

      return {
        ...prev,
        nodes: updatedNodes,
      }
    })
  }

  const handleInsertNode = (node: ThemeNode, index: number) => {
    const nextNodes = [...activeNodes]
    nextNodes.splice(index, 0, node)
    setNodes(nextNodes)
    setSelectedNodeId(node.id)
  }

  const handleRemoveNode = (nodeId: string) => {
    const nextNodes = removeNodeById(activeNodes, nodeId)
    setNodes(nextNodes)
    if (selectedNodeId === nodeId) {
      setSelectedNodeId(null)
    }
  }

  const handleReorderNodes = (fromIndex: number, toIndex: number) => {
    const updated = [...activeNodes]
    const [moved] = updated.splice(fromIndex, 1)
    updated.splice(toIndex, 0, moved)
    setNodes(updated)
  }

  const handleUpdateSelectedNode = (partial: {
    props?: Record<string, unknown>
    layout?: Record<string, unknown>
    size?: Record<string, unknown>
    slots?: Record<string, ThemeNode | null>
  }) => {
    if (!selectedNodeId) return
    const updated = updateNodeById(activeNodes, selectedNodeId, (node) => {
      const nextSlots = { ...(node.slots ?? {}) }
      if (partial.slots) {
        Object.entries(partial.slots).forEach(([key, value]) => {
          if (!value) {
            delete nextSlots[key]
          } else {
            nextSlots[key] = value
          }
        })
      }
      return {
        ...node,
        props: partial.props
          ? {
              ...(node.props ?? {}),
              ...partial.props,
            }
          : node.props,
        layout: partial.layout
          ? {
              ...(node.layout ?? {}),
              ...partial.layout,
            }
          : node.layout,
        size: partial.size
          ? {
              ...(node.size ?? {}),
              ...partial.size,
            }
          : node.size,
        slots: partial.slots ? nextSlots : node.slots,
      }
    })
    setNodes(updated)
  }

  const selectedBlock = selectedNodeId ? findNodeById(activeNodes, selectedNodeId) : null
  const canUndo = history.past.length > 0
  const canRedo = history.future.length > 0

  if (isLoading || isDraftLoading) {
    return (
      <div className="text-muted-foreground flex h-full w-full flex-col items-center justify-center gap-4">
        <Loader2 className="text-primary h-8 w-8 animate-spin" />
        <p>Loading Theme Builder...</p>
      </div>
    )
  }

  if (isError || isDraftError || !data) {
    return (
      <div className="text-destructive flex h-full w-full items-center justify-center">
        Failed to load theme. Please try again.
      </div>
    )
  }

  return (
    <div className="flex h-full w-full flex-col">
      <FluidBuilderHeader
        themeName={data.theme.name}
        pages={availablePages}
        activePageKey={activePageKey}
        onSelectPage={(pageKey) => {
          setActivePageKey(pageKey)
          setSelectedNodeId(null)
        }}
        onCreatePage={handleCreatePage}
        onSave={() => void handleSave()}
        onResetPage={handleResetPage}
        canResetPage={
          data.theme.is_system
            ? Boolean(activePageTemplate)
            : Boolean(activeNode)
        }
        onPublish={() => void handlePublish()}
        actions={
          themeId && realmName ? (
            <HarborResourceActions
              scope="theme"
              id={themeId}
              resourceLabel={data.theme.name}
              invalidateKeys={[
                ['themes', realmName],
                ['themes', realmName, themeId],
                ['themes', realmName, themeId, 'draft'],
                ['themes', realmName, themeId, 'assets'],
                ['themes', realmName, themeId, 'versions'],
                ['theme-pages', realmName, themeId],
                ['theme-template-gaps', realmName, themeId],
                ['theme-bindings', realmName, themeId],
                ['theme-preview', realmName, themeId],
              ]}
            />
          ) : null
        }
        isSaving={isSaving}
        isPublishing={isPublishing}
      />
      {missingTemplates.length > 0 && (
        <div className="border-b px-6 py-2">
          <Alert variant="destructive">
            <AlertTitle>Missing templates in this theme</AlertTitle>
            <AlertDescription className="text-xs">
              Some flows reference templates that are not defined here:{' '}
              {missingTemplates.join(', ')}. Users will fall back to system pages until you add
              them.
            </AlertDescription>
          </Alert>
        </div>
      )}
      <div className="relative flex flex-1 overflow-hidden">
        <FluidPrimarySidebar activePanel={activePanel} onSelectPanel={setActivePanel} />
        {activePanel === 'sections' ? (
          <FluidBlocksPanel
            nodes={activeNodes}
            selectedNodeId={selectedNodeId}
            validationErrors={activeValidationErrors}
            onSelectNode={setSelectedNodeId}
            onInsertNode={handleInsertNode}
            onRemoveNode={handleRemoveNode}
            onReorderNodes={handleReorderNodes}
          />
        ) : (
          <FluidThemeSettingsPanel
            tokens={draftState.tokens}
            layout={draftState.layout}
            assets={assets}
            isUploading={isUploading}
            onTokensChange={(tokens) =>
              commitDraft((prev) => ({
                ...prev,
                tokens,
              }))
            }
            onLayoutChange={(layout) =>
              commitDraft((prev) => ({
                ...prev,
                layout,
              }))
            }
            onUploadAsset={(file) => void uploadAsset(file)}
          />
        )}
        <FluidCanvas
          tokens={draftState.tokens}
          layout={draftState.layout}
          blocks={activeNodes}
          assets={assets}
          selectedNodeId={selectedNodeId}
          isInspecting={isInspecting}
          onSelectNode={setSelectedNodeId}
        />
        <FluidInspector
          assets={assets}
          tokens={draftState.tokens}
          selectedBlock={selectedBlock}
          validationErrors={activeValidationErrors}
          inputNames={inputNames}
          onUpdateSelectedBlock={handleUpdateSelectedNode}
        />
      </div>
      <FluidFloatingActionBar
        isInspecting={isInspecting}
        canUndo={canUndo}
        canRedo={canRedo}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onToggleInspect={() => setIsInspecting((prev) => !prev)}
      />
    </div>
  )
}
