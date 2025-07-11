import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: {
        title: 'Bewässerungsgruppen',
      },
    }
  },
})
