import type { ReactNode } from 'react'
import { describe, it, expect, vi } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { Map as MaplibreMap } from 'maplibre-gl'
import type { ClusterMarkerResponse, RouteResponse } from '@green-ecolution/backend-client'
import { renderHook } from '@/test/utils'
import { wateringPlanQueries } from '@/api/queries'
import { MapContext } from '@/components/map-gl/MapContext'
import { clusterQueries } from '../../api/queries'
import useFitStopsWithoutRoute, { stopBounds } from './useFitStopsWithoutRoute'

const PLAN_ID = '0190c0de-0000-7000-8000-000000000001'

const marker = (id: string, longitude: number, latitude: number) =>
  ({
    id,
    longitude,
    latitude,
    name: id,
    treeCount: 1,
    wateringStatus: 'good',
  }) as ClusterMarkerResponse

const markers = [marker('a', 9.4, 54.7), marker('b', 9.5, 54.8), marker('other', 10, 55)]

const renderFit = (route: RouteResponse | null) => {
  const fitBounds = vi.fn()
  const map = { style: {}, fitBounds } as unknown as MaplibreMap
  const client = new QueryClient()
  client.setQueryData(wateringPlanQueries.route(PLAN_ID).queryKey, route)
  client.setQueryData(clusterQueries.markers().queryKey, { data: markers })
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>
      <MapContext value={map}>{children}</MapContext>
    </QueryClientProvider>
  )
  renderHook(() => useFitStopsWithoutRoute(PLAN_ID, ['a', 'b']), { wrapper })
  return fitBounds
}

describe('useFitStopsWithoutRoute', () => {
  it('frames the plan clusters when the plan has no route', () => {
    const fitBounds = renderFit(null)
    expect(fitBounds).toHaveBeenCalledWith(
      [
        [9.4, 54.7],
        [9.5, 54.8],
      ],
      expect.anything(),
    )
  })

  it('leaves the framing to the core when a route exists', () => {
    const route = {
      geometry: { type: 'LineString', coordinates: [[9.4, 54.7]] },
    } as RouteResponse
    expect(renderFit(route)).not.toHaveBeenCalled()
  })
})

describe('stopBounds', () => {
  it('has no bounds when no stop has a marker', () => {
    expect(stopBounds(markers, ['missing'])).toBeUndefined()
  })
})
