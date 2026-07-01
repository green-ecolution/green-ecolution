import { Suspense } from 'react'
import type { Sensor } from '@/api/backendApi'
import SensorSignalCard from './SensorSignalCard'
import ChartSignalData from './ChartSignalData'

interface SensorSignalSectionProps {
  sensor: Sensor
}

const SensorSignalSection = ({ sensor }: SensorSignalSectionProps) => {
  if (sensor.sensorType !== 'lorawan') return null

  return (
    <>
      <SensorSignalCard sensor={sensor} />
      <Suspense fallback={null}>
        <ChartSignalData sensorId={sensor.id} />
      </Suspense>
    </>
  )
}

export default SensorSignalSection
