import { useMemo } from 'react'
import { type LngLatBoundsLike } from 'maplibre-gl'
import { cn } from '@green-ecolution/ui'
import type { TreeWithDistance } from '@/api/backendApi'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import { useAccuracyRing } from '@/components/map-gl/hooks/useAccuracyRing'
import SelectableTreeMarkers from '@/components/map-gl/SelectableTreeMarkers'

interface NearestTreeMapPreviewProps {
  sensorLat: number
  sensorLng: number
  sensorAccuracy?: number | null
  trees: TreeWithDistance[]
  selectedTreeId: string | null
  onSelectTree?: (treeId: string) => void
  className?: string
}

const NearestTreeLayers = ({
  sensorLat,
  sensorLng,
  sensorAccuracy,
  trees,
  selectedTreeId,
  onSelectTree,
}: Omit<NearestTreeMapPreviewProps, 'className'>) => {
  const map = useMaplibreMap()
  useAccuracyRing(map, 'gec-nearest', sensorLng, sensorLat, sensorAccuracy)
  return (
    <SelectableTreeMarkers
      trees={trees.map((entry) => entry.tree)}
      selectedTreeId={selectedTreeId}
      onSelect={onSelectTree}
    />
  )
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
  const bounds = useMemo<LngLatBoundsLike>(() => {
    const lngs = [sensorLng, ...trees.map((t) => t.tree.longitude)]
    const lats = [sensorLat, ...trees.map((t) => t.tree.latitude)]
    let w = Math.min(...lngs)
    let e = Math.max(...lngs)
    let s = Math.min(...lats)
    let n = Math.max(...lats)
    const padX = (e - w) * 0.3 || 0.001
    const padY = (n - s) * 0.3 || 0.001
    w -= padX
    e += padX
    s -= padY
    n += padY
    return [
      [w, s],
      [e, n],
    ]
  }, [sensorLng, sensorLat, trees])

  return (
    <MapPreview
      bounds={bounds}
      interactive
      ariaLabel="Karte mit Sensor-Position und nahegelegenen Bäumen"
      className={cn('aspect-[4/3] sm:aspect-[16/10]', className)}
    >
      <SensorMarker lng={sensorLng} lat={sensorLat} />
      <NearestTreeLayers
        sensorLat={sensorLat}
        sensorLng={sensorLng}
        sensorAccuracy={sensorAccuracy}
        trees={trees}
        selectedTreeId={selectedTreeId}
        onSelectTree={onSelectTree}
      />
    </MapPreview>
  )
}

export default NearestTreeMapPreview
export type { NearestTreeMapPreviewProps }
