import { useState, useEffect } from 'react'
import { Minus, Plus } from 'lucide-react'
import { useMap } from 'react-leaflet'

const ZoomControls = () => {
  const map = useMap()
  const [canZoomIn, setCanZoomIn] = useState(true)
  const [canZoomOut, setCanZoomOut] = useState(true)

  useEffect(() => {
    const checkZoomLevels = () => {
      const currentZoom = map.getZoom()
      const minZoom = map.getMinZoom()
      const maxZoom = map.getMaxZoom()

      setCanZoomIn(currentZoom < maxZoom)
      setCanZoomOut(currentZoom > minZoom)
    }

    checkZoomLevels()
    map.on('zoomend', checkZoomLevels)

    return () => {
      map.off('zoomend', checkZoomLevels)
    }
  }, [map])

  const handleZoomIn = () => {
    if (canZoomIn) map.zoomIn()
  }

  const handleZoomOut = () => {
    if (canZoomOut) map.zoomOut()
  }

  return (
    <div className="absolute z-[1000] space-y-2 bottom-6 right-4 lg:right-10 lg:bottom-10">
      <button
        type="button"
        className={`bg-white shadow-cards w-10 h-10 rounded-full flex items-center justify-center transition-all ease-in-out duration-300 ${
          !canZoomIn ? 'bg-dark-200 cursor-not-allowed' : ''
        }`}
        onClick={handleZoomIn}
        disabled={!canZoomIn}
      >
        <Plus className="w-6 h-6 text-dark-800" />
      </button>
      <button
        type="button"
        className={`bg-white shadow-cards w-10 h-10 rounded-full flex items-center justify-center transition-all ease-in-out duration-300 ${
          !canZoomOut ? 'bg-dark-200 cursor-not-allowed' : ''
        }`}
        onClick={handleZoomOut}
        disabled={!canZoomOut}
      >
        <Minus className="w-6 h-6 text-dark-800" />
      </button>
    </div>
  )
}

export default ZoomControls
