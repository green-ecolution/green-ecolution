import { vehicleQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/$vehicleId')(
  entityRoute({
    key: 'vehicle',
    query: vehicleQueries.detail,
    idParam: 'vehicleId',
    title: (vehicle) => `Fahrzeug: ${vehicle.numberPlate}`,
    notFound: { entityName: 'Fahrzeug', backTo: '/vehicles', backLabel: 'Zur Fahrzeugliste' },
  }),
)
