import { Suspense } from 'react'
import { useActiveModules } from '@/modules/useActiveModules'

export const MapOverviews = () => {
  const { manifests } = useActiveModules()
  return manifests.map(({ key, map }) => map && <map.Overview key={key} />)
}

// Module slots may be lazy; outside the map's own Suspense they must not blank the page.
export const MapToolbars = () => {
  const { manifests } = useActiveModules()
  return manifests.map(
    ({ key, map }) =>
      map?.Toolbar && (
        <Suspense key={key} fallback={null}>
          <map.Toolbar />
        </Suspense>
      ),
  )
}

export const MapOverlays = () => {
  const { manifests } = useActiveModules()
  return manifests.map(
    ({ key, map }) =>
      map?.Overlay && (
        <Suspense key={key} fallback={null}>
          <map.Overlay />
        </Suspense>
      ),
  )
}
