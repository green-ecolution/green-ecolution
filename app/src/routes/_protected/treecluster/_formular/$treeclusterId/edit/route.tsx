import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/_formular/$treeclusterId/edit')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Bewässerungsgruppe editieren',
      },
    }
  },
})
