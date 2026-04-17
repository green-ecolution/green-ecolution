import { SensorMarkerIcon, TreeMarkerIcon } from '@/components/map/markerIcons'
import ZoomControls from '@/components/map/ZoomControls'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { type TreeWithDistance, WateringStatus } from '@green-ecolution/backend-client'
import { cn } from '@green-ecolution/ui'
import L from 'leaflet'
import { useMemo } from 'react'
import { Circle, MapContainer, Marker, TileLayer } from 'react-leaflet'

interface NearestTreeMapPreviewProps {
  sensorLat: number
  sensorLng: number
  sensorAccuracy?: number | null
  trees: TreeWithDistance[]
  selectedTreeId: number | null
  onSelectTree?: (treeId: number) => void
  className?: string
}

const NearestTreeMapPreview = ({
  sensorLat,
  sensorLng,
  sensorAccuracy,
  trees,
  selectedTreeId,
  onSelectTree,
  className,
}: NearestTreeMapPreviewProps) => {
  const sensorPos = L.latLng(sensorLat, sensorLng)
  const radius = sensorAccuracy && sensorAccuracy > 0 ? sensorAccuracy : null

  const bounds = useMemo(() => {
    const points: L.LatLngExpression[] = [
      [sensorLat, sensorLng],
      ...trees.map((t) => [t.tree.latitude, t.tree.longitude] as L.LatLngTuple),
    ]
    if (points.length < 2) {
      return L.latLngBounds([sensorPos, sensorPos]).pad(0.5)
    }
    return L.latLngBounds(points).pad(0.3)
  }, [sensorLat, sensorLng, sensorPos, trees])

  return (
    <div
      aria-label="Karte mit Sensor-Position und nahegelegenen Bäumen"
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
        dragging={true}
        touchZoom={true}
        doubleClickZoom={true}
        scrollWheelZoom={true}
        boxZoom={true}
        keyboard={true}
        className="z-0 h-full w-full"
        bounds={bounds}
        maxZoom={19}
        minZoom={3}
      >
        <TileLayer url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png" keepBuffer={2} />

        {radius && (
          <Circle
            center={sensorPos}
            radius={radius}
            pathOptions={{
              color: 'oklch(0.62 0.20 145)',
              fillColor: 'oklch(0.62 0.20 145)',
              fillOpacity: 0.15,
              weight: 1.5,
            }}
          />
        )}

        <ZoomControls />
        <Marker position={sensorPos} icon={SensorMarkerIcon()} />

        {trees.map((entry) => {
          const { tree } = entry
          const statusDetails = getWateringStatusDetails(
            tree.wateringStatus ?? WateringStatus.WateringStatusUnknown,
          )
          const isSelected = tree.id === selectedTreeId

          return (
            <Marker
              key={tree.id}
              position={[tree.latitude, tree.longitude]}
              icon={TreeMarkerIcon(statusDetails.colorHex, isSelected, false)}
              eventHandlers={{
                click: () => onSelectTree?.(tree.id),
              }}
            />
          )
        })}
      </MapContainer>

      <span className="pointer-events-none absolute bottom-1 right-2 text-[10px] text-dark-600/80 font-mono bg-white/70 px-1 rounded">
        © OpenStreetMap
      </span>
    </div>
  )
}

export default NearestTreeMapPreview
export type { NearestTreeMapPreviewProps }
