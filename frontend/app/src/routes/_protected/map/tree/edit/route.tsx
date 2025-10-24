import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/map/tree/edit')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: {
        title: 'Baum Standort bearbeiten',
      },
    }
  },
})
