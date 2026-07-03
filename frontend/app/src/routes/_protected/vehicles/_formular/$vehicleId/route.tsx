import { vehicleIdQuery } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId')(
  entityRoute({
    key: 'vehicle',
    query: vehicleIdQuery,
    idParam: 'vehicleId',
    title: (vehicle) => `Fahrzeug ${vehicle.numberPlate}`,
    notFound: { entityName: 'Fahrzeug', backTo: '/vehicles', backLabel: 'Zur Fahrzeugliste' },
  }),
)
