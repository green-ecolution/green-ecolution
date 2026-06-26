import useMapResize from './useMapResize'
import useMapStoreSync from './useMapStoreSync'
import ZoomControls from './ZoomControls'

const MapControls = () => {
  useMapStoreSync()
  useMapResize()
  return <ZoomControls />
}

export default MapControls
