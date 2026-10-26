import { Suspense } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { render, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { z } from 'zod'
import type { Map as MaplibreMap, MapMouseEvent } from 'maplibre-gl'
import { server } from '@/test/mocks/server'
import type { ModuleManifest } from '@/modules/types'
import { MapContext } from './MapContext'

// The real session needs OIDC settings the test environment does not have.
vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

vi.mock('@/modules/registry', () => {
  const probe = (key: string, interactiveLayerIds: string[]): ModuleManifest => ({
    key,
    labelKey: 'common:modules.irrigation',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    map: {
      permissions: [],
      searchSchema: z.object({}),
      interactiveLayerIds,
      Overview: () => null,
    },
  })
  return {
    moduleManifests: [probe('alpha', ['alpha-points']), probe('beta', ['beta-fill'])],
  }
})

const { default: MapBackgroundClick } = await import('./MapBackgroundClick')

const fakeMap = (hits: unknown[]) => {
  let click: ((e: MapMouseEvent) => void) | undefined
  const map = {
    style: {},
    on: vi.fn((_: string, handler: (e: MapMouseEvent) => void) => {
      click = handler
    }),
    off: vi.fn(),
    getLayer: vi.fn(() => ({})),
    queryRenderedFeatures: vi.fn(() => hits),
  }
  return { map, click: () => click?.({ point: { x: 1, y: 2 } } as MapMouseEvent) }
}

const renderWithActive = (active: string[], hits: unknown[] = []) => {
  server.use(
    http.get('*/v1/modules', () =>
      HttpResponse.json({
        modules: ['alpha', 'beta'],
        organizations: [{ organizationId: 'org', active }],
      }),
    ),
  )
  const onBackground = vi.fn()
  const fake = fakeMap(hits)
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  render(
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={null}>
        <MapContext value={fake.map as unknown as MaplibreMap}>
          <MapBackgroundClick onBackground={onBackground} />
        </MapContext>
      </Suspense>
    </QueryClientProvider>,
  )
  return { ...fake, onBackground }
}

describe('MapBackgroundClick', () => {
  it('checks a click against the interactive layers of the active modules only', async () => {
    const { map, click, onBackground } = renderWithActive(['alpha'])
    await waitFor(() => expect(map.on).toHaveBeenCalledWith('click', expect.any(Function)))

    click()

    expect(map.queryRenderedFeatures).toHaveBeenCalledWith(
      { x: 1, y: 2 },
      { layers: ['alpha-points'] },
    )
    expect(onBackground).toHaveBeenCalledTimes(1)
  })

  it('does not report a click that hits a module layer', async () => {
    const { map, click, onBackground } = renderWithActive(['alpha', 'beta'], [{ id: 'feature' }])
    await waitFor(() => expect(map.on).toHaveBeenCalledWith('click', expect.any(Function)))

    click()

    expect(map.queryRenderedFeatures).toHaveBeenCalledWith(
      { x: 1, y: 2 },
      { layers: ['alpha-points', 'beta-fill'] },
    )
    expect(onBackground).not.toHaveBeenCalled()
  })
})
