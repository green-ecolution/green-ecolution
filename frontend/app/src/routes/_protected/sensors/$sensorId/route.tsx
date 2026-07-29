import { sensorQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/$sensorId')(
  entityRoute({
    key: 'sensor',
    query: sensorQueries.detail,
    idParam: 'sensorId',
    title: (sensor) => `Sensor ID: ${sensor.id}`,
    notFound: { entityName: 'Sensor', backTo: '/sensors', backLabel: 'Zur Sensorenliste' },
  }),
)
