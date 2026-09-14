import { createFileRoute, redirect } from '@tanstack/react-router'
import { infoQueries } from '@/api/queries'
import { crumbRoute, forbiddenErrorComponent, requirePermission } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/plugin')({
  ...crumbRoute('plugins'),
  beforeLoad: async (opts) => {
    const services = await opts.context.queryClient.ensureQueryData(infoQueries.services())
    const plugins = services.items.find((item) => item.name === 'plugins')
    if (!plugins?.enabled) {
      throw redirect({ to: '/settings' })
    }
    await requirePermission(['plugin:read'])(opts)
  },
  errorComponent: forbiddenErrorComponent(),
})
