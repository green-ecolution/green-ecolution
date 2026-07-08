import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import useStore from '@/store/store'
import { useSidebarCollapsed } from './useSidebarCollapsed'

const mockMatchMedia = (matches: boolean) => {
  vi.spyOn(window, 'matchMedia').mockImplementation(
    (query: string) =>
      ({
        matches,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }) as unknown as MediaQueryList,
  )
}

describe('useSidebarCollapsed', () => {
  beforeEach(() => {
    useStore.setState({ sidebarCollapsed: null })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('defaults to collapsed below 1280px when no user choice is set', () => {
    mockMatchMedia(false)
    const { result } = renderHook(() => useSidebarCollapsed())
    expect(result.current).toBe(true)
  })

  it('defaults to expanded at 1280px and above when no user choice is set', () => {
    mockMatchMedia(true)
    const { result } = renderHook(() => useSidebarCollapsed())
    expect(result.current).toBe(false)
  })

  it('prefers the explicit user choice over the breakpoint default', () => {
    mockMatchMedia(true)
    useStore.setState({ sidebarCollapsed: true })
    const { result: collapsed } = renderHook(() => useSidebarCollapsed())
    expect(collapsed.current).toBe(true)

    mockMatchMedia(false)
    useStore.setState({ sidebarCollapsed: false })
    const { result: expanded } = renderHook(() => useSidebarCollapsed())
    expect(expanded.current).toBe(false)
  })
})
