import type { Sensor } from '@/api/backendApi'
import SensorSignalCard from './SensorSignalCard'

interface SensorSignalSectionProps {
  sensor: Sensor
}

const SensorSignalSection = ({ sensor }: SensorSignalSectionProps) => {
  if (sensor.sensorType !== 'lorawan') return null

  return <SensorSignalCard sensor={sensor} />
}

export default SensorSignalSection
