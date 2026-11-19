import { createFileRoute } from '@tanstack/react-router'
import PluginsPage from '@/components/settings/plugin/PluginsPage'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/plugins/')({
  component: PluginsPage,
})
