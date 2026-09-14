import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import PluginDetailPage from '@/components/settings/plugin/PluginDetailPage'
import { pendingLoading } from '@/lib/router'

const pluginRoute = getRouteApi('/_protected/settings/plugin/$slug')

export const Route = createFileRoute('/_protected/settings/plugin/$slug/')({
  pendingComponent: pendingLoading({ key: 'settings:plugin.detail.loading' }),
  component: SinglePlugin,
})

function SinglePlugin() {
  const { plugin } = pluginRoute.useLoaderData()

  return <PluginDetailPage plugin={plugin} />
}
