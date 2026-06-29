import { createContext, use } from 'react'
import type { Map as MaplibreMap } from 'maplibre-gl'

export const MapContext = createContext<MaplibreMap | null>(null)

// Children of MapCanvas only render once the map is ready, so this never returns null there.
export function useMaplibreMap(): MaplibreMap {
  const map = use(MapContext)
  if (!map) {
    throw new Error('useMaplibreMap must be used within a ready MapCanvas')
  }
  return map
}
