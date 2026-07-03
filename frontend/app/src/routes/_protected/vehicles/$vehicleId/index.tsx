import VehicleDashboard from '@/components/vehicle/VehicleDashboard'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const vehicleRoute = getRouteApi('/_protected/vehicles/$vehicleId')

export const Route = createFileRoute('/_protected/vehicles/$vehicleId/')({
  pendingComponent: pendingLoading('Fahrzeug wird geladen …'),
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
