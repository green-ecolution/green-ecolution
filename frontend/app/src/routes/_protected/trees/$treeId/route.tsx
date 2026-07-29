import { treeQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/$treeId')(
  entityRoute({
    key: 'tree',
    query: treeQueries.detail,
    idParam: 'treeId',
    title: (tree) => `Baum: ${tree.number}`,
    notFound: { entityName: 'Baum', backTo: '/trees', backLabel: 'Zur Baumliste' },
  }),
)
