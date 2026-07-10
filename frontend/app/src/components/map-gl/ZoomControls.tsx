import { Minus, Plus } from 'lucide-react'
import { useMaplibreMap } from './MapContext'
import MapControlButton from './MapControlButton'

const ZoomControls = () => {
  const map = useMaplibreMap()
  return (
    <>
      <MapControlButton aria-label="Hineinzoomen" onClick={() => map.zoomIn()}>
        <Plus className="!size-6" />
      </MapControlButton>
      <MapControlButton aria-label="Herauszoomen" onClick={() => map.zoomOut()}>
        <Minus className="!size-6" />
      </MapControlButton>
    </>
  )
}

export default ZoomControls
