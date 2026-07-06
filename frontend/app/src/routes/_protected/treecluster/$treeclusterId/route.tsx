import { treeClusterIdQuery } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/$treeclusterId')(
  entityRoute({
    key: 'treecluster',
    query: treeClusterIdQuery,
    idParam: 'treeclusterId',
    title: (treecluster) => treecluster.name,
    notFound: {
      entityName: 'Bewässerungsgruppe',
      backTo: '/treecluster',
      backLabel: 'Zur Gruppenliste',
    },
  }),
)
