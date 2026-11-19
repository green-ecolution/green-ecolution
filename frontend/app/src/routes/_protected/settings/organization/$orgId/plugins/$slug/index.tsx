import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import PluginDetailPage from '@/components/settings/plugin/PluginDetailPage'
import { pendingLoading } from '@/lib/router'

const pluginRoute = getRouteApi('/_protected/settings/organization/$orgId/plugins/$slug')

export const Route = createFileRoute('/_protected/settings/organization/$orgId/plugins/$slug/')({
  pendingComponent: pendingLoading({ key: 'settings:plugin.detail.loading' }),
  component: SinglePlugin,
})

function SinglePlugin() {
  const { plugin } = pluginRoute.useLoaderData()

  return <PluginDetailPage plugin={plugin} />
}
