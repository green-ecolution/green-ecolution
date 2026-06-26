import { useEffect, useRef } from 'react'
import { Marker, type GeoJSONSource } from 'maplibre-gl'
import { cn } from '@green-ecolution/ui'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import MapPreview from '@/components/map-gl/MapPreview'
import { metersCircle } from '@/components/map-gl/metersCircle'
import { isMapAlive } from '@/components/map-gl/mapReady'

interface LocationMapPreviewProps {
  latitude: number
  longitude: number
  /** Accuracy radius in meters — drawn as a circle around the marker. */
  accuracyMeters?: number | null
  /** Tailwind classes applied to the wrapper. */
  className?: string
  /** Initial zoom (default: 18 — close enough to read a city block). */
  zoom?: number
  /** ARIA label for the wrapper. */
  ariaLabel?: string
  /** When `true`, users can pan and pinch-zoom. Default: `false`. */
  interactive?: boolean
  /** Re-center if the position drifts out of the viewport. Default: `true`. */
  follow?: boolean
}

const ACCURACY_SOURCE = 'gec-preview-accuracy'
const ACCURACY_FILL = 'gec-preview-accuracy-fill'
const ACCURACY_LINE = 'gec-preview-accuracy-line'

const PositionLayers = ({
  latitude,
  longitude,
  accuracyMeters,
  follow,
}: Pick<LocationMapPreviewProps, 'latitude' | 'longitude' | 'accuracyMeters' | 'follow'>) => {
  const map = useMaplibreMap()
  const markerRef = useRef<Marker | null>(null)

  useEffect(() => {
    const marker = new Marker({ color: '#486725' }).setLngLat([longitude, latitude]).addTo(map)
    markerRef.current = marker
    if (!map.getSource(ACCURACY_SOURCE)) {
      map.addSource(ACCURACY_SOURCE, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
      map.addLayer({
        id: ACCURACY_FILL,
        type: 'fill',
        source: ACCURACY_SOURCE,
        paint: { 'fill-color': '#486725', 'fill-opacity': 0.15 },
      })
      map.addLayer({
        id: ACCURACY_LINE,
        type: 'line',
        source: ACCURACY_SOURCE,
        paint: { 'line-color': '#486725', 'line-width': 1.5 },
      })
    }
    return () => {
      marker.remove()
      markerRef.current = null
      if (!isMapAlive(map)) return
      for (const id of [ACCURACY_LINE, ACCURACY_FILL]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(ACCURACY_SOURCE)) map.removeSource(ACCURACY_SOURCE)
    }
    // Created once; position updates happen in the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [map])

  useEffect(() => {
    markerRef.current?.setLngLat([longitude, latitude])
    if (!isMapAlive(map)) return
    const features =
      accuracyMeters && accuracyMeters > 0
        ? [metersCircle(longitude, latitude, accuracyMeters)]
        : []
    map.getSource<GeoJSONSource>(ACCURACY_SOURCE)?.setData({ type: 'FeatureCollection', features })
  }, [map, longitude, latitude, accuracyMeters])

  useEffect(() => {
    if (!follow || !isMapAlive(map)) return
    if (!map.getBounds().contains([longitude, latitude])) {
      map.easeTo({ center: [longitude, latitude] })
    }
  }, [map, longitude, latitude, follow])

  return null
}

const LocationMapPreview = ({
  latitude,
  longitude,
  accuracyMeters,
  className,
  zoom = 18,
  ariaLabel = 'Karte mit aktueller GPS-Position',
  interactive = false,
  follow = true,
}: LocationMapPreviewProps) => (
  <MapPreview
    center={[longitude, latitude]}
    zoom={zoom}
    interactive={interactive}
    ariaLabel={ariaLabel}
    className={cn('aspect-[4/3] sm:aspect-[16/10]', className)}
  >
    <PositionLayers
      latitude={latitude}
      longitude={longitude}
      accuracyMeters={accuracyMeters}
      follow={follow}
    />
  </MapPreview>
)

export default LocationMapPreview
