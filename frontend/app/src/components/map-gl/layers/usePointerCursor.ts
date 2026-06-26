import { useEffect } from 'react'
import { useMaplibreMap } from '../MapContext'

export const usePointerCursor = (layerId: string, enabled = true) => {
  const map = useMaplibreMap()
  useEffect(() => {
    if (!enabled) return
    const enter = () => {
      map.getCanvas().style.cursor = 'pointer'
    }
    const leave = () => {
      map.getCanvas().style.cursor = ''
    }
    map.on('mouseenter', layerId, enter)
    map.on('mouseleave', layerId, leave)
    return () => {
      map.off('mouseenter', layerId, enter)
      map.off('mouseleave', layerId, leave)
    }
  }, [map, layerId, enabled])
}
