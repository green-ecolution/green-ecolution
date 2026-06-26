import { useEffect, useRef } from 'react'
import { Marker } from 'maplibre-gl'
import { cn } from '@green-ecolution/ui'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import MapPreview from '@/components/map-gl/MapPreview'
import { useAccuracyRing } from '@/components/map-gl/hooks/useAccuracyRing'
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
    return () => {
      marker.remove()
      markerRef.current = null
    }
    // Created once; position updates happen in the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [map])

  useEffect(() => {
    markerRef.current?.setLngLat([longitude, latitude])
  }, [longitude, latitude])

  useAccuracyRing(map, 'gec-preview', longitude, latitude, accuracyMeters)

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
