import { useSuspenseQuery } from '@tanstack/react-query'
import { userQueries } from '@/api/queries'
import useStore from '@/store/store'
import { DEFAULT_OPEN_ZOOM } from '@/lib/mapConfig'

export interface MapView {
  /** `[lat, lng]`. */
  center: [number, number]
  /**
   * The limits the map is held to, or null where it may be panned and zoomed
   * freely. Box and zoom range travel together: an organization either holds
   * its map to a working area or it does not.
   */
  bounds: {
    bbox: [number, number, number, number]
    minZoom: number
    maxZoom: number
  } | null
}

/**
 * The viewport the map opens at, resolved for the signed-in user's
 * organization. The backend falls back to the instance default wherever no
 * organization applies, so there is no second code path here.
 *
 * Suspends rather than starting at a default and correcting itself: the map is
 * built once, from the final values, which is what keeps it from visibly
 * jumping on the first paint.
 */
const useMapView = (): MapView => {
  const { data } = useSuspenseQuery(userQueries.mapView())
  const [lat, lng] = data.center
  const bounds =
    data.bbox && data.minZoom != null && data.maxZoom != null
      ? {
          bbox: data.bbox as [number, number, number, number],
          minZoom: data.minZoom,
          maxZoom: data.maxZoom,
        }
      : null
  return { center: [lat, lng], bounds }
}

/**
 * Where a secondary map should sit: the live viewport once the user has moved
 * one, otherwise the organization's configured centre. Embedded previews and
 * form maps follow the main map around this way, without needing a position of
 * their own.
 */
export const useMapCenter = (): [number, number] => {
  const stored = useStore((s) => s.mapCenter)
  const { center } = useMapView()
  return stored ?? center
}

/**
 * The zoom a secondary map should open at: the live one once the user has
 * moved a map, otherwise the organization's lower bound, and failing that the
 * level the application opens at when nothing constrains it.
 */
export const useMapZoom = (): number => {
  const stored = useStore((s) => s.mapZoom)
  const { bounds } = useMapView()
  return stored ?? bounds?.minZoom ?? DEFAULT_OPEN_ZOOM
}

export default useMapView
