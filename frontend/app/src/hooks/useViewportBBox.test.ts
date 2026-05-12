import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useViewportBBox, expandBBox, fitsInside } from './useViewportBBox'
import type { BoundingBox } from '@/api/queries'

const fakeMap = (bounds: { south: number; west: number; north: number; east: number }) => {
  const listeners: Record<string, (() => void)[]> = {}
  return {
    getBounds: () => ({
      getSouth: () => bounds.south,
      getWest: () => bounds.west,
      getNorth: () => bounds.north,
      getEast: () => bounds.east,
    }),
    on: (event: string, fn: () => void) => {
      listeners[event] ??= []
      listeners[event].push(fn)
    },
    off: (event: string, fn: () => void) => {
      listeners[event] = (listeners[event] ?? []).filter((f) => f !== fn)
    },
    fire: (event: string) => {
      ;(listeners[event] ?? []).forEach((fn) => fn())
    },
    setBounds: (b: typeof bounds) => {
      bounds = b
    },
  }
}

vi.mock('react-leaflet', () => ({
  useMap: () => fakeMapInstance,
}))

let fakeMapInstance: ReturnType<typeof fakeMap>

beforeEach(() => {
  vi.useFakeTimers()
  fakeMapInstance = fakeMap({ south: 54.78, west: 9.4, north: 54.81, east: 9.46 })
})

afterEach(() => {
  vi.useRealTimers()
})

describe('expandBBox', () => {
  it('scales by the buffer factor around the centre', () => {
    const b: BoundingBox = { swLat: 0, swLng: 0, neLat: 10, neLng: 10 }
    const e = expandBBox(b, 1.5)
    expect(e.swLat).toBeCloseTo(-2.5)
    expect(e.swLng).toBeCloseTo(-2.5)
    expect(e.neLat).toBeCloseTo(12.5)
    expect(e.neLng).toBeCloseTo(12.5)
  })
})

describe('fitsInside', () => {
  it('returns true when inner is contained', () => {
    const outer: BoundingBox = { swLat: 0, swLng: 0, neLat: 10, neLng: 10 }
    const inner: BoundingBox = { swLat: 1, swLng: 1, neLat: 9, neLng: 9 }
    expect(fitsInside(inner, outer)).toBe(true)
  })

  it('returns false when inner overflows', () => {
    const outer: BoundingBox = { swLat: 0, swLng: 0, neLat: 10, neLng: 10 }
    const inner: BoundingBox = { swLat: -1, swLng: 0, neLat: 9, neLng: 9 }
    expect(fitsInside(inner, outer)).toBe(false)
  })
})

describe('useViewportBBox', () => {
  it('returns initial buffered bbox on mount', () => {
    const { result } = renderHook(() => useViewportBBox())
    expect(result.current).not.toBeNull()
    expect(result.current!.swLat).toBeLessThan(54.78)
    expect(result.current!.neLat).toBeGreaterThan(54.81)
  })

  it('does not change when small pan stays inside buffer', () => {
    const { result } = renderHook(() => useViewportBBox())
    const before = result.current
    act(() => {
      fakeMapInstance.setBounds({ south: 54.785, west: 9.405, north: 54.815, east: 9.465 })
      fakeMapInstance.fire('moveend')
      vi.advanceTimersByTime(250)
    })
    expect(result.current).toBe(before)
  })

  it('updates when pan leaves buffer', () => {
    const { result } = renderHook(() => useViewportBBox())
    const before = result.current
    act(() => {
      fakeMapInstance.setBounds({ south: 55.0, west: 9.4, north: 55.1, east: 9.46 })
      fakeMapInstance.fire('moveend')
      vi.advanceTimersByTime(250)
    })
    expect(result.current).not.toBe(before)
    expect(result.current!.swLat).toBeGreaterThan(54.9)
  })
})
