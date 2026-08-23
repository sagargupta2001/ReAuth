import { createContext, useContext } from 'react'

import type { ThemeAsset, ThemeNode } from '@/entities/theme/model/types'
import type { FieldTarget } from '@/features/fluid/model/inspectorFields'

export interface NodePatch {
  props?: Record<string, unknown>
  layout?: Record<string, unknown>
  size?: Record<string, unknown>
  slots?: Record<string, ThemeNode | null>
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
  read: (target: FieldTarget, key: string) => unknown
  /** Merges a single value into the node's props, layout, or size. */
  write: (target: FieldTarget, key: string, value: unknown) => void
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
