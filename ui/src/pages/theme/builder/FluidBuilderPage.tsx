import { useEffect, useMemo, useState } from 'react'

import { Loader2 } from 'lucide-react'
import { useParams } from 'react-router-dom'

import type {
  ThemeBlock,
  ThemeBlueprint,
  ThemeDraft,
  ThemePageTemplate,
} from '@/entities/theme/model/types'
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
import { FluidInspector } from '@/features/fluid/components/FluidInspector'
import { FluidPrimarySidebar } from '@/features/fluid/components/FluidPrimarySidebar'
import { FluidThemeSettingsPanel } from '@/features/fluid/components/FluidThemeSettingsPanel'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'

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

function extractBlocks(blueprint?: ThemeBlueprint): ThemeBlock[] {
  if (!blueprint) return []
  if (Array.isArray(blueprint)) return blueprint
  return blueprint.blocks ?? []
}

function updateBlueprint(blueprint: ThemeBlueprint | undefined, blocks: ThemeBlock[]): ThemeBlueprint {
  if (!blueprint || Array.isArray(blueprint)) {
    return { layout: 'default', blocks }
  }
  return {
    ...blueprint,
    layout: blueprint.layout ?? 'default',
    blocks,
  }
}

export function FluidBuilderPage() {
  const { themeId } = useParams()
  const { data, isLoading, isError } = useTheme(themeId)
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
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null)
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
      setSelectedIndex(null)
    }
  }, [activeDraft])

  useEffect(() => {
    if (availablePages.length === 0) return
    const isValid = availablePages.some((page) => page.key === activePageKey)
    if (!isValid) {
      setActivePageKey(availablePages[0].key)
    }
  }, [activePageKey, availablePages])

  const { mutateAsync: saveDraft, isPending: isSaving } = useSaveThemeDraft(themeId || '')
  const { mutateAsync: publishTheme, isPending: isPublishing } = usePublishTheme(themeId || '')
  const { data: assets = [] } = useThemeAssets(themeId)
  const { mutateAsync: uploadAsset, isPending: isUploading } = useUploadThemeAsset(themeId || '')

  const handleSave = async () => {
    await saveDraft(history.present)
  }

  const handlePublish = async () => {
    await saveDraft(history.present)
    await publishTheme()
  }

  const handleResetPage = () => {
    if (!activePageKey) return
    setSelectedIndex(null)

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
      blocks: [
        {
          block: 'text',
          props: { text: label },
          children: [],
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
    setSelectedIndex(null)
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

  const activeBlocks = useMemo(() => {
    return extractBlocks(activeBlueprint)
  }, [activeBlueprint])

  useEffect(() => {
    if (selectedIndex !== null && selectedIndex >= activeBlocks.length) {
      setSelectedIndex(null)
    }
  }, [activeBlocks.length, selectedIndex])

  useEffect(() => {
    setSelectedIndex(null)
  }, [activePageKey])

  const setBlocks = (blocks: ThemeBlock[]) => {
    commitDraft((prev) => {
      const index = prev.nodes.findIndex((node) => node.node_key === activePageKey)
      const hasNode = index >= 0
      const baseBlueprint = hasNode ? prev.nodes[index].blueprint : activeBlueprint
      const updatedNode = {
        node_key: activePageKey,
        blueprint: updateBlueprint(baseBlueprint, blocks),
      }

      if (!hasNode) {
        return {
          ...prev,
          nodes: [...prev.nodes, updatedNode],
        }
      }

      const nodes = [...prev.nodes]
      nodes[index] = {
        ...nodes[index],
        ...updatedNode,
      }

      return {
        ...prev,
        nodes,
      }
    })
  }

  const handleInsertBlock = (block: ThemeBlock, index: number) => {
    const nextBlocks = [...activeBlocks]
    nextBlocks.splice(index, 0, block)
    setBlocks(nextBlocks)
    setSelectedIndex(index)
  }

  const handleRemoveBlock = (index: number) => {
    const nextBlocks = activeBlocks.filter((_, idx) => idx !== index)
    setBlocks(nextBlocks)
    if (selectedIndex === null) return
    if (selectedIndex === index) {
      setSelectedIndex(null)
    } else if (selectedIndex > index) {
      setSelectedIndex(selectedIndex - 1)
    }
  }

  const handleReorderBlocks = (fromIndex: number, toIndex: number) => {
    const updated = [...activeBlocks]
    const [moved] = updated.splice(fromIndex, 1)
    updated.splice(toIndex, 0, moved)
    setBlocks(updated)

    if (selectedIndex === null) return
    if (selectedIndex === fromIndex) {
      setSelectedIndex(toIndex)
      return
    }
    if (fromIndex < selectedIndex && toIndex >= selectedIndex) {
      setSelectedIndex(selectedIndex - 1)
      return
    }
    if (fromIndex > selectedIndex && toIndex <= selectedIndex) {
      setSelectedIndex(selectedIndex + 1)
    }
  }

  const handleUpdateSelectedBlock = (partial: Record<string, unknown>) => {
    if (selectedIndex === null) return
    const updated = activeBlocks.map((block, index) =>
      index === selectedIndex
        ? {
            ...block,
            props: {
              ...(block.props ?? {}),
              ...partial,
            },
          }
        : block,
    )
    setBlocks(updated)
  }

  const selectedBlock = selectedIndex === null ? null : activeBlocks[selectedIndex] ?? null
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
          setSelectedIndex(null)
        }}
        onCreatePage={handleCreatePage}
        isInspecting={isInspecting}
        onToggleInspect={() => setIsInspecting((prev) => !prev)}
        onUndo={handleUndo}
        onRedo={handleRedo}
        canUndo={canUndo}
        canRedo={canRedo}
        onSave={() => void handleSave()}
        onResetPage={handleResetPage}
        canResetPage={
          data.theme.is_system
            ? Boolean(activePageTemplate)
            : Boolean(activeNode)
        }
        onPublish={() => void handlePublish()}
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
            blocks={activeBlocks}
            selectedIndex={selectedIndex}
            onSelectBlock={setSelectedIndex}
            onInsertBlock={handleInsertBlock}
            onRemoveBlock={handleRemoveBlock}
            onReorderBlocks={handleReorderBlocks}
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
          blocks={activeBlocks}
          assets={assets}
          selectedIndex={selectedIndex}
          isInspecting={isInspecting}
          onSelectBlock={setSelectedIndex}
        />
        <FluidInspector
          assets={assets}
          selectedBlock={selectedBlock}
          onUpdateSelectedBlock={handleUpdateSelectedBlock}
        />
      </div>
    </div>
  )
}
