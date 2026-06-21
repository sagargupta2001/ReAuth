import { useLayoutEffect, useRef } from 'react'

import { getAnimationEngine } from './animation.engine'
import type { BoxSize } from './animation.types'

function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches
}

/**
 * Fluidly morphs the returned element from its previous natural box size to its
 * new one whenever `contentKey` changes — the iOS "Dynamic Island" grow/shrink.
 * Honours `prefers-reduced-motion` by snapping without animation.
 */
export function useFluidResize<T extends HTMLElement>(contentKey: string) {
  const ref = useRef<T>(null)
  const prevSize = useRef<BoxSize | null>(null)
  const prevKey = useRef<string | null>(null)

  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return

    // New content has committed; `el` now reflects its natural size.
    const to: BoxSize = { width: el.offsetWidth, height: el.offsetHeight }
    const from = prevSize.current
    const keyChanged = prevKey.current !== null && prevKey.current !== contentKey

    if (from && keyChanged && !prefersReducedMotion()) {
      void getAnimationEngine().morphSize(el, { from, to })
    }

    prevSize.current = to
    prevKey.current = contentKey
  }, [contentKey])

  return ref
}
