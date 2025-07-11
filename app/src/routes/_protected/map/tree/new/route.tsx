import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/map/tree/new')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: { title: 'Baum Standort erfassen' },
    }
  },
})
