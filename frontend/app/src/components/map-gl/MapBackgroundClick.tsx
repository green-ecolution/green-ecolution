import { useEffect } from 'react'
import type { MapMouseEvent } from 'maplibre-gl'
import { interactiveLayerIdsFor } from '@/modules/map'
import { useActiveModules } from '@/modules/useActiveModules'
import { useMaplibreMap } from './MapContext'
import { isMapAlive } from './mapReady'

interface MapBackgroundClickProps {
  onBackground: () => void
}

// A click counts as background only if it misses the interactive layers of
// every active module, not just those of the module listening.
const MapBackgroundClick = ({ onBackground }: MapBackgroundClickProps) => {
  const map = useMaplibreMap()
  const { manifests } = useActiveModules()
  const interactiveLayers = interactiveLayerIdsFor(manifests)

  useEffect(() => {
    const handler = (e: MapMouseEvent) => {
      if (!isMapAlive(map)) return
      const present = interactiveLayers.filter((id) => map.getLayer(id))
      const hits = present.length ? map.queryRenderedFeatures(e.point, { layers: present }) : []
      if (hits.length === 0) onBackground()
    }
    map.on('click', handler)
    return () => {
      map.off('click', handler)
    }
  }, [map, onBackground, interactiveLayers])

  return null
}

export default MapBackgroundClick
