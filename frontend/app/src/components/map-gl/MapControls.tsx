import useMapResize from './useMapResize'
import useMapStoreSync from './useMapStoreSync'
import ZoomControls from './ZoomControls'

const MapControls = () => {
  useMapStoreSync()
  useMapResize()
  return (
    <div className="absolute bottom-6 right-4 z-[1000] flex flex-col gap-4 lg:bottom-10 lg:right-10">
      <div className="flex flex-col gap-2">
        <ZoomControls />
      </div>
    </div>
  )
}

export default MapControls
