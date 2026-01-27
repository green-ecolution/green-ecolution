import { Loading } from '@green-ecolution/ui'
import VehicleDashboard from '@/components/vehicle/VehicleDashboard'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const vehicleRoute = getRouteApi('/_protected/vehicles/$vehicleId')

export const Route = createFileRoute('/_protected/vehicles/$vehicleId/')({
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Fahrzeug wird geladen …" />
  ),
  component: SingleVehicle,
})

function SingleVehicle() {
  const { vehicle } = vehicleRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <VehicleDashboard vehicle={vehicle} />
    </div>
  )
}

export default SingleVehicle
