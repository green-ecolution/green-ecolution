import { sensorQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { getI18n } from '@/lib/i18n'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/$sensorId')(
  entityRoute({
    key: 'sensor',
    query: sensorQueries.detail,
    idParam: 'sensorId',
    title: (sensor) => getI18n().t('sensor:detail.title', { id: sensor.id }),
    notFound: {
      entityName: { key: 'sensor:entity.name' },
      backTo: '/sensors',
      backLabel: { key: 'sensor:detail.notFoundBackLabel' },
    },
  }),
)
