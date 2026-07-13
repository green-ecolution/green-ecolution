import useStore from '@/store/store'
import {
  createFileRoute,
  Outlet,
  useLocation,
  useNavigate,
  useSearch,
} from '@tanstack/react-router'
import { z } from 'zod'
import MapCanvas from '@/components/map-gl/MapCanvas'
import MapControls from '@/components/map-gl/MapControls'
import MapBackgroundClick from '@/components/map-gl/MapBackgroundClick'
import MapToolbarBar from '@/components/map/MapToolbarBar'
import ClusterPanel from '@/components/map/cluster-panel/ClusterPanel'
import { clusterBoundariesQuery, clusterMarkersQuery } from '@/api/queries'
import { pendingLoading, prefetch } from '@/lib/router'
import { Loading } from '@green-ecolution/ui'
import { Suspense, useCallback, useState } from 'react'

const mapSearchParamsSchema = z.object({
  selected: z.string().optional(),
  lat: z.number().default(useStore.getState().mapCenter[0]).catch(useStore.getState().mapCenter[0]),
  lng: z.number().default(useStore.getState().mapCenter[1]).catch(useStore.getState().mapCenter[1]),
  clusterId: z.string().optional(),
  sensorId: z.string().optional(),
  zoom: z
    .number()
    .int()
    .max(useStore.getState().mapMaxZoom)
    .min(useStore.getState().mapMinZoom)
    .default(useStore.getState().mapMinZoom)
    .catch(useStore.getState().mapMinZoom),
})

export const Route = createFileRoute('/_protected/map')({
  component: MapRoot,
  validateSearch: mapSearchParamsSchema,
  loaderDeps: ({ search: { lat, lng, zoom } }) => ({
    lat,
    lng,
    zoom,
  }),
  loader: ({ context: { queryClient }, deps: { lat, lng, zoom } }) => {
    prefetch(queryClient, clusterMarkersQuery(), 'clusterMarkersQuery')
    prefetch(queryClient, clusterBoundariesQuery(), 'clusterBoundariesQuery')

    useStore.setState({ mapCenter: [lat, lng], mapZoom: zoom })

    return {
      crumb: { title: 'Karte' },
    }
  },
  pendingComponent: pendingLoading('Lade Karte...'),
})

function MapRoot() {
  const { pathname } = useLocation()
  const navigate = useNavigate()
  const search = useSearch({ strict: false })
  // Reactive exact-match: matchRoute() didn't reliably re-render MapRoot on
  // pathname-only changes, leaving isIndex stale across /map ↔ sub-route nav.
  const isIndex = pathname === '/map' || pathname === '/map/'
  const panelClusterId = isIndex ? search.cluster : undefined

  const [snapPoint, setSnapPoint] = useState<number | string | null>('260px')

  const handleClosePanel = useCallback(() => {
    setSnapPoint('260px')
    navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: undefined }) }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }, [navigate])

  const handleEditCluster = useCallback(() => {
    if (!panelClusterId) return
    navigate({
      to: '/map/treecluster/edit/$treeclusterId',
      params: { treeclusterId: panelClusterId },
      search: (prev) => prev,
    }).catch((error) => console.error('Navigation failed:', error))
  }, [navigate, panelClusterId])

  const handleOpenDashboard = useCallback(() => {
    if (!panelClusterId) return
    navigate({
      to: '/treecluster/$treeclusterId',
      params: { treeclusterId: panelClusterId },
    }).catch((error) => console.error('Navigation failed:', error))
  }, [navigate, panelClusterId])

  return (
    <div className="flex h-[calc(100dvh-4.563rem)] flex-col">
      {isIndex && <MapToolbarBar />}
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <div className="relative flex-1">
          <Suspense fallback={<Loading className="mt-20 justify-center" label="Lade Karte..." />}>
            <MapCanvas>
              <MapControls />
              {isIndex && <MapBackgroundClick onBackground={handleClosePanel} />}
              <Suspense
                fallback={<Loading className="mt-20 justify-center" label="Lade Karte..." />}
              >
                <Outlet />
              </Suspense>
            </MapCanvas>
          </Suspense>
          {panelClusterId && (
            <ClusterPanel
              key={panelClusterId}
              clusterId={panelClusterId}
              onClose={handleClosePanel}
              onOpenDashboard={handleOpenDashboard}
              onEdit={handleEditCluster}
              activeSnapPoint={snapPoint}
              setActiveSnapPoint={setSnapPoint}
            />
          )}
        </div>
      </div>
    </div>
  )
}
