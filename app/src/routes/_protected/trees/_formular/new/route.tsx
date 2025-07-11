import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/_formular/new')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: {
        title: 'Neuer Baum',
      },
    }
  },
})
