import { useEffect } from 'react'
import { useMap } from 'react-leaflet'

const MapResizeHandler = () => {
  const map = useMap()

  useEffect(() => {
    const observer = new ResizeObserver(() => map.invalidateSize())
    observer.observe(map.getContainer())
    return () => observer.disconnect()
  }, [map])

  return null
}

export default MapResizeHandler
