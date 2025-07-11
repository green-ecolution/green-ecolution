import { Marker } from 'react-leaflet'
import { RefillIcon, RouteIcon } from '../MapMarker'

export interface RouteMarkerProps {
  label?: string
  type: 'route' | 'refill'
  position: { latitude: number; longitude: number }
}

const RouteMarker = ({ label, type, position }: RouteMarkerProps) => {
  return (
    <Marker
      icon={type === 'route' ? RouteIcon(label ?? 'Route') : RefillIcon()}
      position={[position.latitude, position.longitude]}
    />
  )
}

export default RouteMarker
