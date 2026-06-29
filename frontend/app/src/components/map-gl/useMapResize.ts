import { useEffect } from 'react'
import { useMaplibreMap } from './MapContext'

const useMapResize = () => {
  const map = useMaplibreMap()
  useEffect(() => {
    const observer = new ResizeObserver(() => map.resize())
    observer.observe(map.getContainer())
    return () => observer.disconnect()
  }, [map])
}

export default useMapResize
