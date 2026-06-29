import { Minus, Plus } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import { useMaplibreMap } from './MapContext'

const ZoomControls = () => {
  const map = useMaplibreMap()
  return (
    <div className="absolute z-[1000] flex flex-col gap-2 bottom-6 right-4 lg:right-10 lg:bottom-10">
      <Button
        variant="outline"
        size="icon"
        onClick={() => map.zoomIn()}
        className="rounded-full shadow-cards bg-white border-0"
      >
        <Plus className="!size-6 text-dark-800" />
      </Button>
      <Button
        variant="outline"
        size="icon"
        onClick={() => map.zoomOut()}
        className="rounded-full shadow-cards bg-white border-0"
      >
        <Minus className="!size-6 text-dark-800" />
      </Button>
    </div>
  )
}

export default ZoomControls
