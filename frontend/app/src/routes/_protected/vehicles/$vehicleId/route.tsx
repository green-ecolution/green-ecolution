import { vehicleQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/$vehicleId')(
  entityRoute({
    key: 'vehicle',
    query: vehicleQueries.detail,
    idParam: 'vehicleId',
    title: (vehicle) => ({
      titleKey: 'vehicle:detail.title',
      params: { numberPlate: vehicle.numberPlate },
    }),
    notFound: {
      entityName: { key: 'vehicle:entity.name' },
      backTo: '/vehicles',
      backLabel: { key: 'vehicle:detail.notFoundBackLabel' },
    },
  }),
)
