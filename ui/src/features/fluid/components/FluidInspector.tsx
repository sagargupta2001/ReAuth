import { useCallback, useEffect, useMemo, useState } from 'react'

import { Button } from '@/components/button'
import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import type { ThemeValidationError } from '@/features/fluid/lib/themeValidation'
import { Alert, AlertDescription, AlertTitle } from '@/shared/ui/alert'
import { SectionCard } from '@/shared/ui/section-card'
import { contrastRatio } from '@/shared/lib/colorUtils'
import {
  createActionId,
  withActionIds,
  type InspectorAction,
} from '@/features/fluid/lib/actionBindings'
import { resolveNodeStyle, resolvePartStyle } from '@/features/fluid/lib/nodeStyle'
import {
  FieldTarget,
  matchesNode,
  type InspectorSection,
} from '@/features/fluid/model/inspectorFields'
import { INSPECTOR_SECTIONS } from '@/features/fluid/model/inspectorSchema'
import { ActionsPanel } from './inspector/ActionsPanel'
import { ContrastCard } from './inspector/ContrastCard'
import { InspectorSectionCard } from './inspector/InspectorSectionCard'
import {
  InspectorProvider,
  type InspectorContextValue,
  type NodePatch,
} from './inspector/inspectorContext'

/** Context paths offered when mapping an action payload. */
const CONTEXT_SUGGESTIONS = [
  'context.username',
  'context.email',
  'context.client_id',
  'context.realm',
  'context.resume_token',
  'context.action_type',
] as const

/** Components that can emit signals into the auth flow. */
const ACTION_CAPABLE_COMPONENTS = ['button', 'input']

function supportsActions(node: ThemeNode | null): boolean {
  if (!node) return false
  if (node.type === 'Input') return true
  if (node.type !== 'Component') return false
  return ACTION_CAPABLE_COMPONENTS.includes((node.component ?? '').toLowerCase())
}

interface FluidInspectorProps {
  assets: ThemeAsset[]
  tokens: Record<string, unknown>
  selectedBlock: ThemeNode | null
  validationErrors?: ThemeValidationError[]
  inputNames: string[]
  onUpdateSelectedBlock: (partial: NodePatch) => void
}

/**
 * Right sidebar for the selected node.
 *
 * The Properties tab renders `INSPECTOR_SECTIONS` filtered by node type, so this
 * component holds no per-property knowledge. Only the Actions tab and the
 * contrast report are bespoke.
 */
export function FluidInspector({
  assets,
  tokens,
  selectedBlock,
  validationErrors = [],
  inputNames,
  onUpdateSelectedBlock,
}: FluidInspectorProps) {
  const [activeTab, setActiveTab] = useState<'properties' | 'actions'>('properties')
  const canUseActions = supportsActions(selectedBlock)
  const resolvedTab = canUseActions ? activeTab : 'properties'

  const selectedProps = useMemo(() => selectedBlock?.props ?? {}, [selectedBlock])

  const colorTokens = useMemo(() => {
    const raw = tokens?.colors
    return raw && typeof raw === 'object' && !Array.isArray(raw)
      ? (raw as Record<string, unknown>)
      : {}
  }, [tokens])
  const textContrast = useMemo(() => {
    const background = String(colorTokens.background || colorTokens.surface || '#ffffff')
    const color = String(selectedProps.color || colorTokens.text || '#111827')
    return contrastRatio(color, background)
  }, [colorTokens, selectedProps.color])

  const selectedErrors = useMemo(
    () =>
      selectedBlock
        ? validationErrors.filter((error) => error.nodeId === selectedBlock.id)
        : [],
    [selectedBlock, validationErrors],
  )

  const actions = useMemo<InspectorAction[]>(() => {
    const raw = (selectedProps as Record<string, unknown>).actions
    if (!Array.isArray(raw)) return []
    return withActionIds(
      raw.filter((action) => action && typeof action === 'object') as InspectorAction[],
    )
  }, [selectedProps])

  const updateActions = useCallback(
    (next: InspectorAction[]) => {
      if (!selectedBlock) return
      onUpdateSelectedBlock({ props: { actions: withActionIds(next) } })
    },
    [selectedBlock, onUpdateSelectedBlock],
  )

  // Backfill missing ids once, so React keys and patches stay stable.
  useEffect(() => {
    const raw = (selectedProps as Record<string, unknown>).actions
    if (!Array.isArray(raw) || raw.length === 0) return
    const needsIds = raw.some(
      (action) => action && typeof action === 'object' && !(action as InspectorAction).action_id,
    )
    if (!needsIds) return
    onUpdateSelectedBlock({
      props: {
        actions: raw.map((action) => ({
          ...(action as InspectorAction),
          action_id: (action as InspectorAction).action_id ?? createActionId(),
        })),
      },
    })
  }, [selectedProps, onUpdateSelectedBlock])

  const inspector = useMemo<InspectorContextValue | null>(() => {
    if (!selectedBlock) return null
    // `layout` and `size` are typed shapes; reading them by key needs an index
    // signature, and the schema is what constrains which keys are valid.
    const records: Record<string, Record<string, unknown>> = {
      [FieldTarget.Props]: selectedBlock.props ?? {},
      [FieldTarget.Layout]: (selectedBlock.layout ?? {}) as Record<string, unknown>,
      [FieldTarget.Size]: (selectedBlock.size ?? {}) as Record<string, unknown>,
    }
    return {
      node: selectedBlock,
      assets,
      read: (target, key, address) => {
        if (target !== FieldTarget.Style) return records[target][key]
        if (!address?.group) return undefined
        const resolved = address.part
          ? resolvePartStyle(selectedBlock, address.part)
          : resolveNodeStyle(selectedBlock)
        return (resolved[address.group] as Record<string, unknown>)[key]
      },
      write: (target, key, value, address) => {
        if (target !== FieldTarget.Style) {
          onUpdateSelectedBlock({ [target]: { [key]: value } })
          return
        }
        if (!address?.group) return
        onUpdateSelectedBlock({
          style: { group: address.group, key, part: address.part, value },
        })
      },
      patch: onUpdateSelectedBlock,
    }
  }, [selectedBlock, assets, onUpdateSelectedBlock])

  const sections = useMemo<InspectorSection[]>(
    () =>
      selectedBlock
        ? INSPECTOR_SECTIONS.filter((section) => matchesNode(section.appliesTo, selectedBlock))
        : [],
    [selectedBlock],
  )

  return (
    <aside className="bg-muted/10 flex w-80 flex-col border-l">
      <div className="bg-background border-b px-4 py-3">
        <h3 className="text-sm font-semibold">Inspector</h3>
      </div>

      <div className="flex-1 space-y-4 overflow-y-auto p-4">
        {!selectedBlock || !inspector ? (
          <SectionCard title="Element" description="Nothing selected.">
            <p className="text-muted-foreground text-sm">
              Select a block from the canvas to edit its properties.
            </p>
          </SectionCard>
        ) : (
          <InspectorProvider value={inspector}>
            {selectedErrors.length > 0 && (
              <Alert variant="destructive">
                <AlertTitle>Validation</AlertTitle>
                <AlertDescription>
                  <div className="space-y-1">
                    {selectedErrors.map((error, index) => (
                      <div key={`${error.path}-${index}`}>{error.message}</div>
                    ))}
                  </div>
                </AlertDescription>
              </Alert>
            )}

            {canUseActions && (
              <div className="flex gap-2">
                <Button
                  size="sm"
                  variant={resolvedTab === 'properties' ? 'default' : 'outline'}
                  className="h-8 flex-1 px-3 text-xs"
                  onClick={() => setActiveTab('properties')}
                >
                  Properties
                </Button>
                <Button
                  size="sm"
                  variant={resolvedTab === 'actions' ? 'default' : 'outline'}
                  className="h-8 flex-1 px-3 text-xs"
                  onClick={() => setActiveTab('actions')}
                >
                  Actions
                </Button>
              </div>
            )}

            {resolvedTab === 'properties' ? (
              <>
                {sections.map((section) => (
                  <InspectorSectionCard key={section.id} section={section} />
                ))}
                {selectedBlock.type === 'Text' && <ContrastCard ratio={textContrast} />}
              </>
            ) : (
              <SectionCard title="Actions" description="Signals this block emits.">
                <ActionsPanel
                  actions={actions}
                  inputNames={inputNames}
                  contextSuggestions={[...CONTEXT_SUGGESTIONS]}
                  onChange={updateActions}
                />
              </SectionCard>
            )}
          </InspectorProvider>
        )}
      </div>
    </aside>
  )
}
