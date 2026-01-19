import useStore from '@/store/store'
import { createFileRoute, Outlet } from '@tanstack/react-router'
import { z } from 'zod'
import Map from '@/components/map/Map'
import MapController from '@/components/map/MapController'
import ZoomControls from '@/components/map/ZoomControls'
import { treeClusterQuery, treeQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import { Suspense } from 'react'

const mapSearchParamsSchema = z.object({
  selected: z.string().optional(),
  lat: z.number().catch(useStore.getState().mapCenter[0]),
  lng: z.number().catch(useStore.getState().mapCenter[1]),
  clusterId: z.number().optional(),
  sensorId: z.string().optional(),
  zoom: z
    .number()
    .int()
    .max(useStore.getState().mapMaxZoom)
    .min(useStore.getState().mapMinZoom)
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
      .prefetchQuery(treeClusterQuery())
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))
    queryClient
      .prefetchQuery(treeQuery())
      .catch((error) => console.error('Prefetching "treeQuery" failed:', error))

    useStore.setState({ mapCenter: [lat, lng], mapZoom: zoom })

    return {
      crumb: { title: 'Karte' },
    }
  },
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Lade Karte..." />,
})

function MapRoot() {
  return (
    <div className="relative">
      <Map>
        <MapController />
        <ZoomControls />
        <Suspense fallback={<Loading className="mt-20 justify-center" label="Lade Karte..." />}>
          <Outlet />
        </Suspense>
      </Map>
    </div>
  )
}
