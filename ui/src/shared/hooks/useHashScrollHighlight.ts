import { useEffect } from 'react'
import { useLocation } from 'react-router-dom'

import { getAnimationEngine } from '@/lib/animations/animation.engine'

const HIGHLIGHT_CLASS = 'animate-target-highlight'
const HIGHLIGHT_DURATION_MS = 2000

function getHashTarget(): string | null {
  const rawHash = window.location.hash
  if (!rawHash) return null

  const lastHashIndex = rawHash.lastIndexOf('#')
  if (lastHashIndex <= 0) return null

  const target = rawHash.slice(lastHashIndex + 1).trim()
  return target ? decodeURIComponent(target) : null
}

export function useHashScrollHighlight() {
  const location = useLocation()

  useEffect(() => {
    const targetId = getHashTarget()
    if (!targetId) return

    const element = document.getElementById(targetId)
    if (!element) return

    element.scrollIntoView({ behavior: 'smooth', block: 'center' })

    element.classList.remove(HIGHLIGHT_CLASS)
    void element.offsetWidth
    element.classList.add(HIGHLIGHT_CLASS)

    getAnimationEngine().highlight(element, { duration: 1.6, hold: 0.2 })

    const timeout = window.setTimeout(() => {
      element.classList.remove(HIGHLIGHT_CLASS)
    }, HIGHLIGHT_DURATION_MS)

    return () => window.clearTimeout(timeout)
  }, [location.key, location.pathname, location.search, location.hash])
}
