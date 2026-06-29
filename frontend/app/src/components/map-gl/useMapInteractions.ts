import { useCallback } from 'react'
import { useMaplibreMap } from './MapContext'

const useMapInteractions = () => {
  const map = useMaplibreMap()

  const enableDragging = useCallback(() => {
    map.dragPan.enable()
    map.scrollZoom.enable()
    map.doubleClickZoom.enable()
  }, [map])

  const disableDragging = useCallback(() => {
    map.dragPan.disable()
    map.scrollZoom.disable()
    map.doubleClickZoom.disable()
  }, [map])

  return { enableDragging, disableDragging }
}

export default useMapInteractions
