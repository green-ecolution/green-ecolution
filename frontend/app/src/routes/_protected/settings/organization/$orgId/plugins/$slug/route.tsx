import { createFileRoute } from '@tanstack/react-router'
import { pluginQuery } from '@/api/queries'
import { entityRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/plugins/$slug')(
  entityRoute({
    key: 'plugin',
    query: pluginQuery,
    idParam: 'slug',
    title: (plugin) => plugin.name,
    notFound: {
      entityName: { key: 'settings:plugin.entityName' },
      backTo: '/settings/organization',
      backLabel: { key: 'settings:plugin.notFoundBackLabel' },
    },
  }),
)
