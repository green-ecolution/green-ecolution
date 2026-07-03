import { sensorIdQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import SensorDashboard from '@/components/sensor/SensorDashboard'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/$sensorId/')({
  component: SingleSensor,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Sensoren werden geladen …" />
  ),
  loader: ({ params, context }) =>
    context.queryClient.prefetchQuery(sensorIdQuery(params.sensorId)),
})

function SingleSensor() {
  const sensorId = Route.useParams().sensorId
  const { data: sensor } = useSuspenseQuery(sensorIdQuery(sensorId))

  return (
    <div className="container mt-6">
      <SensorDashboard sensor={sensor} />
    </div>
  )
}
