import { Marker, Tooltip } from 'react-leaflet'
import { Sensor } from '@green-ecolution/backend-client'
import { SensorMarkerIcon } from '../MapMarker'

export interface SensorMarkerProps {
  sensor: Sensor
}

const SensorMarker = ({ sensor }: SensorMarkerProps) => {
  return (
    <Marker icon={SensorMarkerIcon()} position={[sensor.latitude, sensor.longitude]}>
      {sensor.id && (
        <Tooltip direction="top" offset={[5, -40]} className="font-nunito-sans font-semibold">
          {sensor.id}
        </Tooltip>
      )}
    </Marker>
  )
}

export default SensorMarker
