import { Suspense } from 'react'
import type { Sensor } from '@/api/backendApi'
import { useActiveModules } from '@/modules/useActiveModules'

/** Every active module's insights, whatever the sensor is linked to. */
const SensorInsights = ({ sensor }: { sensor: Sensor }) => {
  const { manifests } = useActiveModules()
  return manifests.map(({ key, sensorInsights: Insights }) =>
    Insights ? (
      <Suspense key={key} fallback={null}>
        <Insights sensor={sensor} />
      </Suspense>
    ) : null,
  )
}

export default SensorInsights
