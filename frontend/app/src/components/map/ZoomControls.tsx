import { useState, useEffect } from 'react'
import { Minus, Plus } from 'lucide-react'
import { useMap } from 'react-leaflet'
import { Button } from '@green-ecolution/ui'

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
    <div className="absolute z-[1000] flex flex-col gap-2 bottom-6 right-4 lg:right-10 lg:bottom-10">
      <Button
        variant="outline"
        size="icon"
        onClick={handleZoomIn}
        disabled={!canZoomIn}
        className="rounded-full shadow-cards bg-white"
      >
        <Plus className="!size-6 text-dark-800" />
      </Button>
      <Button
        variant="outline"
        size="icon"
        onClick={handleZoomOut}
        disabled={!canZoomOut}
        className="rounded-full shadow-cards bg-white"
      >
        <Minus className="!size-6 text-dark-800" />
      </Button>
    </div>
  )
}

export default ZoomControls
