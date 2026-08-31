import { useCallback, useState, type RefCallback } from 'react'

interface ContainerWiderThan<T extends HTMLElement> {
  ref: RefCallback<T>
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
  const [isWide, setIsWide] = useState(false)

  // A callback ref, not useLayoutEffect + useRef: pages render a loading state
  // before the layout they measure, so the element is absent on the first commit
  // and an effect that only depends on minWidth would never measure it.
  const ref = useCallback<RefCallback<T>>(
    (node) => {
      if (!node) return

      setIsWide(node.getBoundingClientRect().width >= minWidth)

      const observer = new ResizeObserver(([entry]) => {
        if (entry) setIsWide(entry.contentRect.width >= minWidth)
      })
      observer.observe(node)
      return () => observer.disconnect()
    },
    [minWidth],
  )

  return { ref, isWide }
}
