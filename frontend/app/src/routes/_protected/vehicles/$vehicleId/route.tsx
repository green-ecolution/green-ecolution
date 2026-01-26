import { vehicleIdQuery } from '@/api/queries'
import EntityNotFound from '@/components/layout/EntityNotFound'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/$vehicleId')({
  component: Outlet,
  loader: async ({ context: { queryClient }, params: { vehicleId } }) => {
    const vehicle = await queryClient.fetchQuery(vehicleIdQuery(vehicleId))
    return {
      vehicle,
      crumb: {
        title: `Fahrzeug: ${vehicle.numberPlate}`,
      },
    }
  },
  errorComponent: () => (
    <EntityNotFound entityName="Fahrzeug" backTo="/vehicles" backLabel="Zur Fahrzeugliste" />
  ),
})
