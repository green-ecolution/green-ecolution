import { treeClusterIdQuery } from '@/api/queries'
import EntityNotFound from '@/components/layout/EntityNotFound'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/$treeclusterId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params: { treeclusterId } }) => {
    const treecluster = await queryClient.fetchQuery(treeClusterIdQuery(treeclusterId))
    return {
      treecluster,
      crumb: {
        title: treecluster.name,
      },
    }
  },
  errorComponent: () => (
    <EntityNotFound
      entityName="Bewässerungsgruppe"
      backTo="/treecluster"
      backLabel="Zur Gruppenliste"
    />
  ),
})
