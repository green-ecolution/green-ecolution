import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: {
        title: 'Bäume',
      },
    }
  },
})
