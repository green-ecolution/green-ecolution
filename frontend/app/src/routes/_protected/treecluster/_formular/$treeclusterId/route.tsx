import { treeClusterIdQuery } from '@/api/queries'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/_formular/$treeclusterId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params }) => {
    const cluster = await queryClient.fetchQuery(treeClusterIdQuery(params.treeclusterId))
    return {
      crumb: {
        title: cluster.name,
      },
    }
  },
})
