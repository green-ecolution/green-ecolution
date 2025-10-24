import { createFileRoute, Outlet } from '@tanstack/react-router'
import { sensorIdQuery } from '@/api/queries'

export const Route = createFileRoute('/_protected/sensors/$sensorId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params }) => {
    const sensor = await queryClient.fetchQuery(sensorIdQuery(params.sensorId))
    return {
      crumb: {
        title: `Sensor ID: ${sensor.id}`,
      },
    }
  },
})
