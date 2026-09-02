import { sensorQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/$sensorId')(
  entityRoute({
    key: 'sensor',
    query: sensorQueries.detail,
    idParam: 'sensorId',
    title: (sensor) => ({ titleKey: 'sensor:detail.title', params: { id: sensor.id } }),
    notFound: {
      entityName: { key: 'sensor:entity.name' },
      backTo: '/sensors',
      backLabel: { key: 'sensor:detail.notFoundBackLabel' },
    },
  }),
)
