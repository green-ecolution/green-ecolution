import { vehicleIdQuery } from '@/api/queries'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params }) => {
    const vehicle = await queryClient.fetchQuery(vehicleIdQuery(params.vehicleId))
    return {
      crumb: {
        title: 'Fahrzeug ' + vehicle.numberPlate,
      },
    }
  },
})
