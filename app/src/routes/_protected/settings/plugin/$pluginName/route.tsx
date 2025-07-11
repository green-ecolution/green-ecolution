import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/settings/plugin/$pluginName')({
  component: Outlet,
  loader: ({ params: { pluginName } }) => {
    return {
      crumb: {
        title: pluginName,
      },
    }
  },
})
