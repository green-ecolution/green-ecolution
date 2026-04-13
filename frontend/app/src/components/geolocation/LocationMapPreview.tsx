import defaultIconPng from 'leaflet/dist/images/marker-icon.png'
import shadowIconPng from 'leaflet/dist/images/marker-shadow.png'
import L, { Icon } from 'leaflet'
import { useEffect } from 'react'
import { Circle, MapContainer, Marker, TileLayer, useMap } from 'react-leaflet'
import { cn } from '@green-ecolution/ui'

const markerIcon = new Icon({
  iconUrl: defaultIconPng,
  shadowUrl: shadowIconPng,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  shadowSize: [41, 41],
})

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
  /**
   * When `true`, users can pan and pinch-zoom. Default: `false` — the preview
   * stays locked on the reported position, which is the right default for
   * onboarding summaries where the map is an anchor, not a tool.
   */
  interactive?: boolean
  /**
   * Keep the marker centered as the position updates. When `true`, re-centers
   * only if the new position drifts out of the current viewport, so a user
   * who panned manually isn't yanked back to center on every update.
   * Default: `true`.
   */
  follow?: boolean
}

const FollowPosition = ({ latitude, longitude }: { latitude: number; longitude: number }) => {
  const map = useMap()
  useEffect(() => {
    const next = L.latLng(latitude, longitude)
    if (map.getBounds().contains(next)) return
    map.panTo(next, { animate: true })
  }, [map, latitude, longitude])
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
}: LocationMapPreviewProps) => {
  const center = L.latLng(latitude, longitude)
  const radius = accuracyMeters && accuracyMeters > 0 ? accuracyMeters : null

  return (
    <div
      // A dragging/zooming container isn't an `img` — set role only when locked.
      role={interactive ? undefined : 'img'}
      aria-label={ariaLabel}
      className={cn(
        'relative w-full overflow-hidden rounded-2xl border border-dark-100 shadow-cards',
        'aspect-[4/3] sm:aspect-[16/10]',
        className,
      )}
    >
      <MapContainer
        preferCanvas
        zoomControl={false}
        attributionControl={false}
        dragging={interactive}
        touchZoom={interactive}
        doubleClickZoom={interactive}
        scrollWheelZoom={interactive}
        boxZoom={interactive}
        keyboard={interactive}
        className="z-0 h-full w-full"
        center={center}
        zoom={zoom}
        maxZoom={19}
        minZoom={3}
      >
        <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" keepBuffer={2} />
        {radius && (
          <Circle
            center={center}
            radius={radius}
            pathOptions={{
              color: 'oklch(0.62 0.20 145)',
              fillColor: 'oklch(0.62 0.20 145)',
              fillOpacity: 0.15,
              weight: 1.5,
            }}
          />
        )}
        <Marker position={center} icon={markerIcon} />
        {follow && <FollowPosition latitude={latitude} longitude={longitude} />}
      </MapContainer>
      <span className="pointer-events-none absolute bottom-1 right-2 text-[10px] text-dark-600/80 font-mono bg-white/70 px-1 rounded">
        © OpenStreetMap
      </span>
    </div>
  )
}

export default LocationMapPreview
