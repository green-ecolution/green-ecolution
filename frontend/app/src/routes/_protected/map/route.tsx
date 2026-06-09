import useStore from '@/store/store'
import {
  createFileRoute,
  Outlet,
  useMatchRoute,
  useNavigate,
  useSearch,
} from '@tanstack/react-router'
import { z } from 'zod'
import Map from '@/components/map/Map'
import MapController from '@/components/map/MapController'
import ZoomControls from '@/components/map/ZoomControls'
import MapResizeHandler from '@/components/map/MapResizeHandler'
import MapBackgroundClick from '@/components/map/MapBackgroundClick'
import MapToolbarBar from '@/components/map/MapToolbarBar'
import ClusterPanel from '@/components/map/cluster-panel/ClusterPanel'
import { clusterBoundariesQuery, clusterMarkersQuery } from '@/api/queries'
import { Drawer, DrawerContent, DrawerTitle, Loading } from '@green-ecolution/ui'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { Suspense, useCallback } from 'react'

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
    queryClient
      .prefetchQuery(clusterMarkersQuery())
      .catch((error) => console.error('Prefetching "clusterMarkersQuery" failed:', error))

    queryClient
      .prefetchQuery(clusterBoundariesQuery())
      .catch((error) => console.error('Prefetching "clusterBoundariesQuery" failed:', error))

    useStore.setState({ mapCenter: [lat, lng], mapZoom: zoom })

    return {
      crumb: { title: 'Karte' },
    }
  },
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Lade Karte..." />,
})

function MapRoot() {
  const matchRoute = useMatchRoute()
  const navigate = useNavigate()
  const search = useSearch({ strict: false })
  const isDesktop = useMediaQuery('(min-width: 1024px)')
  const isIndex = !!matchRoute({ to: '/map', fuzzy: false })
  const panelClusterId = isIndex ? search.cluster : undefined

  const handleClosePanel = useCallback(() => {
    navigate({ to: '/map', search: (prev) => ({ ...prev, cluster: undefined }) }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }, [navigate])

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
      <div className="flex min-h-0 flex-1">
        <div className="relative flex-1">
          <Map height="100%">
            <MapController />
            <ZoomControls />
            <MapResizeHandler />
            <MapBackgroundClick onBackgroundClick={handleClosePanel} />
            <Suspense fallback={<Loading className="mt-20 justify-center" label="Lade Karte..." />}>
              <Outlet />
            </Suspense>
          </Map>
        </div>
        {isDesktop && panelClusterId && (
          <aside className="w-[28rem] shrink-0 border-l border-dark-100 bg-white">
            <ClusterPanel
              key={panelClusterId}
              clusterId={panelClusterId}
              onClose={handleClosePanel}
              onOpenDashboard={handleOpenDashboard}
            />
          </aside>
        )}
      </div>

      {!isDesktop && (
        <Drawer
          open={!!panelClusterId}
          onOpenChange={(open) => {
            if (!open) handleClosePanel()
          }}
          modal={false}
          snapPoints={['260px', 1]}
        >
          <DrawerContent showOverlay={false}>
            <DrawerTitle className="sr-only">Baumgruppen-Details</DrawerTitle>
            {panelClusterId && (
              <div className="min-h-0 flex-1 overflow-hidden">
                <ClusterPanel
                  key={panelClusterId}
                  clusterId={panelClusterId}
                  onClose={handleClosePanel}
                  onOpenDashboard={handleOpenDashboard}
                />
              </div>
            )}
          </DrawerContent>
        </Drawer>
      )}
    </div>
  )
}
