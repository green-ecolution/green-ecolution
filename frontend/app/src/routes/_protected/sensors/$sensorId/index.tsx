import { sensorIdQuery } from '@/api/queries'
import SensorDashboard from '@/components/sensor/SensorDashboard'
import { pendingLoading } from '@/lib/router'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const sensorRoute = getRouteApi('/_protected/sensors/$sensorId')

export const Route = createFileRoute('/_protected/sensors/$sensorId/')({
  component: SingleSensor,
  pendingComponent: pendingLoading('Sensoren werden geladen …'),
})

function SingleSensor() {
  const { sensorId } = sensorRoute.useParams()
  // Live query instead of loader data: sensor status changes via MQTT and
  // must refresh on invalidation/window focus, like the treecluster dashboard.
  const { data: sensor } = useSuspenseQuery(sensorIdQuery(sensorId))

  return (
    <div className="container mt-6">
      <SensorDashboard sensor={sensor} />
    </div>
  )
}
