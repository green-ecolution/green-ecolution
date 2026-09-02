import VehicleDashboard from '@/components/vehicle/VehicleDashboard'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const vehicleRoute = getRouteApi('/_protected/vehicles/$vehicleId')

export const Route = createFileRoute('/_protected/vehicles/$vehicleId/')({
  pendingComponent: pendingLoading({ key: 'vehicle:detail.loadingLabel' }),
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
