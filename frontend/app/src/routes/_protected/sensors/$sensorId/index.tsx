import { sensorIdQuery, treeSensorIdQuery } from '@/api/queries'
import LoadingInfo from '@/components/general/error/LoadingInfo'
import SensorDashboard from '@/components/sensor/SensorDashboard'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/$sensorId/')({
  component: SingleSensor,
  pendingComponent: () => <LoadingInfo label="Sensoren werden geladen …" />,
  loader: ({ params, context }) =>
    context.queryClient.prefetchQuery(sensorIdQuery(params.sensorId)),
})

function SingleSensor() {
  const sensorId = Route.useParams().sensorId
  const { data: sensor } = useSuspenseQuery(sensorIdQuery(sensorId))
  // Ensure tree query doesn't throw error if it's not available since it's optional
  const { data: linkedTree } = useQuery(treeSensorIdQuery(sensor.id))

  return (
    <div className="container mt-6">
      <SensorDashboard sensor={sensor} sensorTree={linkedTree} />
    </div>
  )
}

export default SingleSensor
