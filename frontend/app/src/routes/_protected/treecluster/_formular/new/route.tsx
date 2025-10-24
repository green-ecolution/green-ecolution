import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/_formular/new')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Neue Bewässerungsgruppe',
      },
    }
  },
})
