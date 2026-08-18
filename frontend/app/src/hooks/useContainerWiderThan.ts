import { useLayoutEffect, useRef, useState, type RefObject } from 'react'

interface ContainerWiderThan<T extends HTMLElement> {
  ref: RefObject<T | null>
  isWide: boolean
}

// Measures the element itself rather than the viewport: a collapsible app sidebar
// (see useSidebarCollapsed) shifts the space available to page content by ~260px
// independently of window size, so no viewport media query can tell two-pane layouts
// from cramped ones. jsdom never runs layout, so this always reports `false` in tests
// (see test/setup.ts's ResizeObserver mock) — tests must mock this hook directly.
export function useContainerWiderThan<T extends HTMLElement = HTMLDivElement>(
  minWidth: number,
): ContainerWiderThan<T> {
  const ref = useRef<T | null>(null)
  const [isWide, setIsWide] = useState(false)

  useLayoutEffect(() => {
    const el = ref.current
    if (!el) return

    setIsWide(el.getBoundingClientRect().width >= minWidth)

    const observer = new ResizeObserver(([entry]) => {
      if (entry) setIsWide(entry.contentRect.width >= minWidth)
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [minWidth])

  return { ref, isWide }
}
