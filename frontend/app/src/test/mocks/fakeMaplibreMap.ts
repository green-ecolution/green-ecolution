import { vi } from 'vitest'

type Handler = (...args: unknown[]) => void

export const createFakeMap = () => {
  const handlers = new Map<string, Set<Handler>>()
  const sources = new Map<string, { setData: ReturnType<typeof vi.fn> }>()
  const layers = new Set<string>()
  let bearing = 0
  let pitch = 0
  let boundsContains = true

  const fire = (event: string, ...args: unknown[]) => {
    handlers.get(event)?.forEach((cb) => cb(...args))
  }

  const map = {
    style: {},
    on: vi.fn((event: string, cb: Handler) => {
      if (!handlers.has(event)) handlers.set(event, new Set())
      handlers.get(event)?.add(cb)
    }),
    off: vi.fn((event: string, cb: Handler) => {
      handlers.get(event)?.delete(cb)
    }),
    fire,
    getBearing: () => bearing,
    setBearing: (b: number) => {
      bearing = b
      fire('rotate')
    },
    getPitch: () => pitch,
    setPitch: (p: number) => {
      pitch = p
      fire('pitch')
    },
    easeTo: vi.fn(),
    getZoom: () => 13,
    setBoundsContains: (v: boolean) => {
      boundsContains = v
    },
    getMaxBounds: vi.fn(() => ({ contains: () => boundsContains })),
    addSource: vi.fn((id: string) => {
      sources.set(id, { setData: vi.fn() })
    }),
    getSource: vi.fn((id: string) => sources.get(id)),
    addLayer: vi.fn((layer: { id: string }) => {
      layers.add(layer.id)
    }),
    getLayer: vi.fn((id: string) => (layers.has(id) ? { id } : undefined)),
    removeLayer: vi.fn((id: string) => {
      layers.delete(id)
    }),
    removeSource: vi.fn((id: string) => {
      sources.delete(id)
    }),
    zoomIn: vi.fn(),
    zoomOut: vi.fn(),
  }
  return map
}

export type FakeMap = ReturnType<typeof createFakeMap>
