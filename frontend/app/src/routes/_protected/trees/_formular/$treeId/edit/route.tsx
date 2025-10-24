import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/_formular/$treeId/edit')({
  component: () => <Outlet />,
  loader: () => {
    return {
      crumb: {
        title: 'Baum editieren',
      },
    }
  },
})
