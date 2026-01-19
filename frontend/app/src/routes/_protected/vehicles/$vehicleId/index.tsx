import { vehicleIdQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import VehicleDashboard from '@/components/vehicle/VehicleDashboard'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/$vehicleId/')({
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Fahrzeug wird geladen …" />
  ),
  component: SingleVehicle,
  loader: ({ context: { queryClient }, params: { vehicleId } }) =>
    queryClient.prefetchQuery(vehicleIdQuery(vehicleId)),
})

function SingleVehicle() {
  const vehicleId = Route.useParams().vehicleId
  const { data: vehicle } = useSuspenseQuery(vehicleIdQuery(vehicleId))

  return (
    <div className="container mt-6">
      <VehicleDashboard vehicle={vehicle} />
    </div>
  )
}

export default SingleVehicle
