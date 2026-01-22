import { useCallback, useEffect, useRef, useState } from 'react'

interface UseScrollSpyOptions {
  offset?: number
  threshold?: number
}

export function useScrollSpy(ids: string[], options: UseScrollSpyOptions = {}) {
  const { offset = 100, threshold = 50 } = options

  // 1. Lazy Initialization
  // Only set default if ids are present.
  const [activeId, setActiveId] = useState<string>('')

  // 2. Stable Refs
  const containerRef = useRef<HTMLDivElement>(null)
  const sectionRefs = useRef<Record<string, HTMLElement | null>>({})
  const isClickScrolling = useRef(false)

  // 3. Sync Active ID with IDs list (The Fix for Flickering)
  useEffect(() => {
    // If we have IDs but no active selection (initial load), select the first one.
    if (ids.length > 0 && activeId === '') {
        setActiveId(ids[0])
        return
    }

    // If the currently selected ID no longer exists in the new list (e.g. data changed),
    // fallback to the first item to prevent "ghost" selections.
    if (ids.length > 0 && activeId !== '' && !ids.includes(activeId)) {
        setActiveId(ids[0])
    }
  }, [ids, activeId])

  // 4. Scroll Handler
  const handleScroll = useCallback(() => {
    if (isClickScrolling.current || !containerRef.current) return

    const container = containerRef.current
    const { scrollTop, scrollHeight, clientHeight } = container

    // A. Bottom Detection (If we hit bottom, force select last item)
    if (Math.abs(scrollHeight - clientHeight - scrollTop) < threshold) {
      const lastId = ids[ids.length - 1]
      if (activeId !== lastId) setActiveId(lastId)
      return
    }

    // B. Standard Intersection Logic
    let currentId = activeId // Default to current to prevent jitter

    // We iterate backwards to find the first section that has crossed our "vision line"
    // This is often more stable than iterating forwards.
    for (let i = 0; i < ids.length; i++) {
        const id = ids[i]
        const element = sectionRefs.current[id]

        if (element) {
            // We consider a section "active" if its top is above our view line (scrollTop + offset)
            // AND its bottom is below that line.
            const elementTop = element.offsetTop
            const elementBottom = elementTop + element.offsetHeight
            const viewLine = scrollTop + offset

            if (viewLine >= elementTop && viewLine < elementBottom) {
                currentId = id
                break // Found it, stop looking
            }
        }
    }

    if (currentId !== activeId) setActiveId(currentId)
  }, [ids, activeId, offset, threshold])

  // 5. Click Handler
  const scrollToSection = useCallback((id: string) => {
    setActiveId(id)
    isClickScrolling.current = true

    sectionRefs.current[id]?.scrollIntoView({ behavior: 'smooth', block: 'start' })

    // Unlock after animation
    setTimeout(() => {
      isClickScrolling.current = false
    }, 700)
  }, [])

  const registerSection = (id: string) => (el: HTMLElement | null) => {
    sectionRefs.current[id] = el
  }

  return {
    activeId,
    containerRef,
    onScroll: handleScroll,
    scrollToSection,
    registerSection,
  }
}
