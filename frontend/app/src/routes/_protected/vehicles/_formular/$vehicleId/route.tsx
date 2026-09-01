import { vehicleQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { getI18n } from '@/lib/i18n'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId')(
  entityRoute({
    key: 'vehicle',
    query: vehicleQueries.detail,
    idParam: 'vehicleId',
    title: (vehicle) => getI18n().t('vehicle:update.crumbTitle', { numberPlate: vehicle.numberPlate }),
    notFound: {
      entityName: { key: 'vehicle:entity.name' },
      backTo: '/vehicles',
      backLabel: { key: 'vehicle:detail.notFoundBackLabel' },
    },
  }),
)
