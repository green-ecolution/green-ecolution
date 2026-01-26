import { Loading } from '@green-ecolution/ui'
import useStore from '@/store/store'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import VehicleUpdate from '@/components/vehicle/VehicleUpdate'

const vehicleFormRoute = getRouteApi('/_protected/vehicles/_formular/$vehicleId')

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId/edit/')({
  component: EditVehicle,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Fahrzeug wird geladen …" />
  ),
  loader: () => {
    if (!useStore.getState().isAuthenticated) return
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
