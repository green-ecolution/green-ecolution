import { createFileRoute, Outlet, redirect } from '@tanstack/react-router'
import { servicesInfoQuery } from '@/api/queries'

export const Route = createFileRoute('/_protected/settings/plugin')({
  component: Outlet,
  beforeLoad: async ({ context }) => {
    const services = await context.queryClient.ensureQueryData(servicesInfoQuery())
    const plugins = services.items.find((item) => item.name === 'plugins')
    if (!plugins?.enabled) {
      throw redirect({ to: '/settings' })
    }
  },
  loader: () => {
    return {
      crumb: {
        title: 'Plugins',
      },
    }
  },
})
