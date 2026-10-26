import { clusterQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/$treeclusterId')(
  entityRoute({
    key: 'treecluster',
    query: clusterQueries.detail,
    idParam: 'treeclusterId',
    title: (treecluster) => treecluster.name,
    notFound: {
      entityName: { key: 'treecluster:entity.name' },
      backTo: '/treecluster',
      backLabel: { key: 'treecluster:detail.notFoundBackLabel' },
    },
  }),
)
