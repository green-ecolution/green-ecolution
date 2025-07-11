import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/map/sensor/select/tree')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: {
        title: 'Vegetation verlinken',
      },
    }
  },
})
