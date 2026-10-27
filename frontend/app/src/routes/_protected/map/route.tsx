import useStore from '@/store/store'
import { createFileRoute, Outlet, useLocation } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import MapCanvas from '@/components/map-gl/MapCanvas'
import MapControls from '@/components/map-gl/MapControls'
import { MapOverlays, MapToolbars } from '@/components/map/MapModuleSlots'
import { modulesQuery } from '@/api/queries'
import { mapPermissionsFor } from '@/lib/auth/routePermissions'
import { forbiddenErrorComponent, pendingLoading, requirePermission } from '@/lib/router'
import { activeManifestsFrom } from '@/modules/activeModules'
import { mapSearchSchemaFor, type MapSearch } from '@/modules/map'
import { moduleManifests } from '@/modules/registry'
import { Loading } from '@green-ecolution/ui'
import { Suspense } from 'react'

// Every registered module's keys, active or not: a shared link keeps its
// parameters even before the active modules are known.
const mapSearchSchema = mapSearchSchemaFor(moduleManifests)

export const Route = createFileRoute('/_protected/map')({
  component: MapRoot,
  validateSearch: (search: Record<string, unknown>): MapSearch => mapSearchSchema.parse(search),
  beforeLoad: async ({ context }) => {
    const modules = await context.queryClient.ensureQueryData(modulesQuery())
    const mapManifests = activeManifestsFrom(moduleManifests, modules).filter(({ map }) => map)
    await requirePermission(mapPermissionsFor(mapManifests))({ context })
    return { mapManifests }
  },
  errorComponent: forbiddenErrorComponent(),
  loaderDeps: ({ search: { lat, lng, zoom } }) => ({
    lat,
    lng,
    zoom,
  }),
  loader: ({ context: { queryClient, mapManifests }, deps: { lat, lng, zoom } }) => {
    for (const { map } of mapManifests) map?.prefetch?.(queryClient)

    if (lat != null && lng != null) useStore.setState({ mapCenter: [lat, lng] })
    if (zoom != null) useStore.setState({ mapZoom: zoom })

    return {
      crumb: { titleKey: 'map' as const },
    }
  },
  pendingComponent: pendingLoading({ key: 'map:root.loadingLabel' }),
})

function MapRoot() {
  const { t } = useTranslation('map')
  const { pathname } = useLocation()
  // Reactive exact-match: matchRoute() didn't reliably re-render MapRoot on
  // pathname-only changes, leaving isIndex stale across /map ↔ sub-route nav.
  const isIndex = pathname === '/map' || pathname === '/map/'

  return (
    // data-fullscreen-map sizes the page to the viewport, see site.css.
    <div data-fullscreen-map className="relative flex min-h-0 flex-1 flex-col">
      {isIndex && <MapToolbars />}
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <div className="relative flex-1">
          <Suspense
            fallback={<Loading className="mt-20 justify-center" label={t('root.loadingLabel')} />}
          >
            <MapCanvas>
              <MapControls />
              <Suspense
                fallback={
                  <Loading className="mt-20 justify-center" label={t('root.loadingLabel')} />
                }
              >
                <Outlet />
              </Suspense>
            </MapCanvas>
          </Suspense>
          <MapOverlays />
        </div>
      </div>
    </div>
  )
}
