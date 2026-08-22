import type { CSSProperties } from 'react'

import type { ThemeNodeLayout } from '@/entities/theme/model/types'

/** Cross-axis alignment for a Box, defaulting to `stretch`. */
export function alignItemsFor(layout: ThemeNodeLayout): CSSProperties['alignItems'] {
  switch (layout.align) {
    case 'center':
      return 'center'
    case 'end':
      return 'flex-end'
    case 'start':
      return 'flex-start'
    case 'baseline':
      return 'baseline'
    default:
      return 'stretch'
  }
}

/**
 * Main-axis distribution for a Box.
 *
 * Returns undefined when unset so the browser default (`flex-start`) applies.
 */
export function justifyContentFor(
  layout: ThemeNodeLayout,
): CSSProperties['justifyContent'] {
  switch (layout.justify) {
    case 'center':
      return 'center'
    case 'end':
      return 'flex-end'
    case 'start':
      return 'flex-start'
    case 'between':
      return 'space-between'
    case 'around':
      return 'space-around'
    default:
      return undefined
  }
}
