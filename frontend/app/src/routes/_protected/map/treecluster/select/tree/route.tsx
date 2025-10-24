import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/map/treecluster/select/tree')({
  component: Outlet,
  loader: () => {
    return { crumb: { title: 'Bäume auswählen' } }
  },
})
