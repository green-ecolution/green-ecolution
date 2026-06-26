import { useEffect, useMemo, useRef } from 'react'
import { Marker, type GeoJSONSource, type LngLatBoundsLike } from 'maplibre-gl'
import { cn } from '@green-ecolution/ui'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { TreeWithDistance } from '@/api/backendApi'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import { metersCircle } from '@/components/map-gl/metersCircle'
import { isMapAlive } from '@/components/map-gl/mapReady'

interface NearestTreeMapPreviewProps {
  sensorLat: number
  sensorLng: number
  sensorAccuracy?: number | null
  trees: TreeWithDistance[]
  selectedTreeId: string | null
  onSelectTree?: (treeId: string) => void
  className?: string
}

const ACCURACY_SOURCE = 'gec-nearest-accuracy'
const ACCURACY_FILL = 'gec-nearest-accuracy-fill'
const ACCURACY_LINE = 'gec-nearest-accuracy-line'

const buildTreeElement = (colorHex: string, isSelected: boolean, isAssigned: boolean) => {
  const el = document.createElement('div')
  el.style.width = '20px'
  el.style.height = '20px'
  el.style.borderRadius = '9999px'
  el.style.background = colorHex
  el.style.border = '2px solid #ffffff'
  el.style.boxShadow = isSelected ? '0 0 0 3px #486725' : '0 1px 3px rgba(0,0,0,0.4)'
  el.style.opacity = isAssigned ? '0.45' : '1'
  el.style.cursor = isAssigned ? 'default' : 'pointer'
  return el
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
  const markersRef = useRef<Marker[]>([])
  const onSelectRef = useRef(onSelectTree)
  useEffect(() => {
    onSelectRef.current = onSelectTree
  })

  useEffect(() => {
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
      if (!isMapAlive(map)) return
      for (const id of [ACCURACY_LINE, ACCURACY_FILL]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(ACCURACY_SOURCE)) map.removeSource(ACCURACY_SOURCE)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    const features =
      sensorAccuracy && sensorAccuracy > 0
        ? [metersCircle(sensorLng, sensorLat, sensorAccuracy)]
        : []
    map.getSource<GeoJSONSource>(ACCURACY_SOURCE)?.setData({ type: 'FeatureCollection', features })
  }, [map, sensorLng, sensorLat, sensorAccuracy])

  useEffect(() => {
    for (const m of markersRef.current) m.remove()
    markersRef.current = trees.map((entry) => {
      const { tree } = entry
      const { colorHex } = getWateringStatusDetails(tree.wateringStatus ?? WateringStatus.Unknown)
      const isAssigned = tree.sensor != null
      const isSelected = !isAssigned && tree.id === selectedTreeId
      const el = buildTreeElement(colorHex, isSelected, isAssigned)
      if (!isAssigned) {
        el.addEventListener('click', () => onSelectRef.current?.(tree.id))
      }
      return new Marker({ element: el }).setLngLat([tree.longitude, tree.latitude]).addTo(map)
    })
    return () => {
      for (const m of markersRef.current) m.remove()
      markersRef.current = []
    }
  }, [map, trees, selectedTreeId])

  return null
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
