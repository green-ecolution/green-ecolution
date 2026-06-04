import { Loading } from '@green-ecolution/ui'
import { getAuthSession } from '@/lib/auth/session'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import VehicleUpdate from '@/components/vehicle/VehicleUpdate'

const vehicleFormRoute = getRouteApi('/_protected/vehicles/_formular/$vehicleId')

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId/edit/')({
  component: EditVehicle,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Fahrzeug wird geladen …" />
  ),
  loader: async () => {
    if (!(await getAuthSession().isAuthenticated())) return
  },
})

function EditVehicle() {
  const { vehicle } = vehicleFormRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <VehicleUpdate vehicleId={vehicle.id.toString()} />
    </div>
  )
}
