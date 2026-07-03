import { treeIdQuery } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/$treeId')(
  entityRoute({
    key: 'tree',
    query: treeIdQuery,
    idParam: 'treeId',
    title: (tree) => `Baum: ${tree.number}`,
    notFound: { entityName: 'Baum', backTo: '/trees', backLabel: 'Zur Baumliste' },
  }),
)
