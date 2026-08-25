import { createContext, useContext } from 'react'

import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import type { StyleEdit } from '@/features/fluid/lib/nodeStyle'
import type { FieldTarget } from '@/features/fluid/model/inspectorFields'

export interface NodePatch {
  props?: Record<string, unknown>
  layout?: Record<string, unknown>
  size?: Record<string, unknown>
  slots?: Record<string, ThemeNode | null>
  /**
   * One grouped-style change. Sent as an intent rather than a merged object so
   * the builder page can apply it against the real node — which is what lets it
   * drop the legacy prop the group replaces.
   */
  style?: StyleEdit
}

/** Which style group, and optionally which component part, a field addresses. */
export interface StyleAddress {
  group?: StyleEdit['group']
  part?: string
}

/**
 * What every inspector field needs to read and write the selected node.
 *
 * Fields are rendered from a schema, so they cannot take tailored props — they
 * pull from here instead, the same way the theme settings fields do.
 */
export interface InspectorContextValue {
  node: ThemeNode
  /** Reads the current value for a field's target and key. */
  read: (target: FieldTarget, key: string, address?: StyleAddress) => unknown
  /** Merges a single value into the node's props, layout, size, or style. */
  write: (
    target: FieldTarget,
    key: string,
    value: unknown,
    address?: StyleAddress,
  ) => void
  /** For patches that do not fit a single target/key, e.g. slots. */
  patch: (patch: NodePatch) => void
  assets: ThemeAsset[]
}

const InspectorContext = createContext<InspectorContextValue | null>(null)

export const InspectorProvider = InspectorContext.Provider

export function useInspector(): InspectorContextValue {
  const value = useContext(InspectorContext)
  if (!value) {
    throw new Error('useInspector must be used inside <InspectorProvider>.')
  }
  return value
}
