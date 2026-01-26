import { treeIdQuery } from '@/api/queries'
import EntityNotFound from '@/components/layout/EntityNotFound'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/$treeId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params: { treeId } }) => {
    const tree = await queryClient.fetchQuery(treeIdQuery(treeId))
    return {
      tree,
      crumb: {
        title: `Baum: ${tree.number}`,
      },
    }
  },
  errorComponent: () => (
    <EntityNotFound entityName="Baum" backTo="/trees" backLabel="Zur Baumliste" />
  ),
})
