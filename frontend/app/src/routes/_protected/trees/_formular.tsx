import { sensorQuery, treeClusterQuery } from '@/api/queries'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/_formular')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient, auth } }) => {
    if (!(await auth.isAuthenticated())) return

    queryClient
      .prefetchQuery(treeClusterQuery())
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))
    queryClient
      .prefetchQuery(sensorQuery())
      .catch((error) => console.error('Prefetching "sensorQuery" failed:', error))
  },
})
