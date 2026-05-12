import { Marker, Tooltip } from 'react-leaflet'
import type { Sensor } from '@/api/backendApi'
import { SensorMarkerIcon } from '../markerIcons'

export interface SensorMarkerProps {
  sensor: Sensor
}

const SensorMarker = ({ sensor }: SensorMarkerProps) => {
  if (!sensor.coordinate) return null
  return (
    <Marker
      icon={SensorMarkerIcon()}
      position={[sensor.coordinate.latitude, sensor.coordinate.longitude]}
    >
      {sensor.id && (
        <Tooltip direction="top" offset={[5, -40]} className="font-nunito-sans font-semibold">
          {sensor.id}
        </Tooltip>
      )}
    </Marker>
  )
}

export default SensorMarker
