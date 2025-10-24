import { sensorQuery, treeClusterQuery } from '@/api/queries'
import useStore from '@/store/store'
import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/_formular')({
  component: () => <Outlet />,
  loader: ({ context: { queryClient } }) => {
    if (!useStore.getState().auth.isAuthenticated) return

    queryClient
      .prefetchQuery(treeClusterQuery())
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))
    queryClient
      .prefetchQuery(sensorQuery())
      .catch((error) => console.error('Prefetching "sensorQuery" failed:', error))
  },
})
