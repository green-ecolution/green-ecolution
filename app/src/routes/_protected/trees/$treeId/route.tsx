import { treeIdQuery } from '@/api/queries'
import { createFileRoute, Outlet } from '@tanstack/react-router'

const RouteComponent = () => <Outlet />

export const Route = createFileRoute('/_protected/trees/$treeId')({
  component: RouteComponent,
  loader: async ({ context: { queryClient }, params: { treeId } }) => {
    const tree = await queryClient.fetchQuery(treeIdQuery(treeId))
    return {
      crumb: {
        title: `Baum: ${tree.number}`,
      },
    }
  },
})
