import { useMapEvents } from 'react-leaflet'

interface MapBackgroundClickProps {
  onBackgroundClick: () => void
}

const MapBackgroundClick = ({ onBackgroundClick }: MapBackgroundClickProps) => {
  useMapEvents({
    click: () => onBackgroundClick(),
  })
  return null
}

export default MapBackgroundClick
