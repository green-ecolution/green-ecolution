import { createFileRoute, redirect } from '@tanstack/react-router'
import { infoQueries } from '@/api/queries'
import { crumbRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/plugin')({
  ...crumbRoute('Plugins'),
  beforeLoad: async ({ context }) => {
    const services = await context.queryClient.ensureQueryData(infoQueries.services())
    const plugins = services.items.find((item) => item.name === 'plugins')
    if (!plugins?.enabled) {
      throw redirect({ to: '/settings' })
    }
  },
})
