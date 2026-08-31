import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { act, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { useContainerWiderThan } from './useContainerWiderThan'

const MIN_WIDTH = 900

let observed: { element: Element; callback: ResizeObserverCallback }[] = []

class TrackingResizeObserver {
  constructor(private callback: ResizeObserverCallback) {}
  observe(element: Element) {
    observed.push({ element, callback: this.callback })
  }
  unobserve = vi.fn()
  disconnect() {
    observed = observed.filter((entry) => entry.callback !== this.callback)
  }
}

const resizeTo = (width: number) => {
  const entry = observed[observed.length - 1]
  if (!entry) throw new Error('nothing is being observed')
  act(() => {
    entry.callback(
      [{ target: entry.element, contentRect: { width } } as unknown as ResizeObserverEntry],
      {} as ResizeObserver,
    )
  })
}

const mockWidth = (width: number) => {
  vi.spyOn(Element.prototype, 'getBoundingClientRect').mockReturnValue({ width } as DOMRect)
}

// The measured element only appears once `mounted` flips, mirroring a page that
// renders a loading state before the layout it measures.
const Probe = ({ mountedFromStart }: { mountedFromStart: boolean }) => {
  const { ref, isWide } = useContainerWiderThan<HTMLDivElement>(MIN_WIDTH)
  const [mounted, setMounted] = useState(mountedFromStart)

  if (!mounted) {
    return <button onClick={() => setMounted(true)}>laden</button>
  }

  return (
    <div ref={ref} data-testid="layout">
      {isWide ? 'two-pane' : 'drawer'}
    </div>
  )
}

describe('useContainerWiderThan', () => {
  beforeEach(() => {
    observed = []
    globalThis.ResizeObserver = TrackingResizeObserver
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('reports wide when the measured element is present in the first render', () => {
    mockWidth(1200)

    render(<Probe mountedFromStart />)

    expect(screen.getByTestId('layout')).toHaveTextContent('two-pane')
  })

  it('reports wide when the measured element mounts after the first render', () => {
    mockWidth(1200)

    render(<Probe mountedFromStart={false} />)
    act(() => {
      screen.getByRole('button', { name: 'laden' }).click()
    })

    expect(screen.getByTestId('layout')).toHaveTextContent('two-pane')
  })

  it('switches back to the narrow layout when the element shrinks', () => {
    mockWidth(1200)

    render(<Probe mountedFromStart />)
    expect(screen.getByTestId('layout')).toHaveTextContent('two-pane')

    resizeTo(600)

    expect(screen.getByTestId('layout')).toHaveTextContent('drawer')
  })
})
