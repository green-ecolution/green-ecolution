import { useEffect } from 'react'
import type { MapMouseEvent } from 'maplibre-gl'
import { useMaplibreMap } from './MapContext'
import { LAYERS } from './mapStyle'

interface MapBackgroundClickProps {
  onBackground: () => void
}

const INTERACTIVE_LAYERS = [LAYERS.treePoints, LAYERS.clusterMarkers, LAYERS.boundaryFill]

const MapBackgroundClick = ({ onBackground }: MapBackgroundClickProps) => {
  const map = useMaplibreMap()

  useEffect(() => {
    const handler = (e: MapMouseEvent) => {
      const present = INTERACTIVE_LAYERS.filter((id) => map.getLayer(id))
      const hits = present.length ? map.queryRenderedFeatures(e.point, { layers: present }) : []
      if (hits.length === 0) onBackground()
    }
    map.on('click', handler)
    return () => {
      map.off('click', handler)
    }
  }, [map, onBackground])

  return null
}

export default MapBackgroundClick
