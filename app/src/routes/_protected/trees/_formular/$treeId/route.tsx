import { treeIdQuery } from '@/api/queries'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/_formular/$treeId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params: { treeId } }) => {
    const tree = await queryClient.fetchQuery(treeIdQuery(treeId))
    return {
      crumb: {
        title: `Baum: ${tree.number}`,
      },
    }
  },
})
