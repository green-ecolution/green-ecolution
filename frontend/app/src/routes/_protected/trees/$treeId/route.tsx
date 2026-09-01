import { treeQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'
import { getI18n } from '@/lib/i18n'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/$treeId')(
  entityRoute({
    key: 'tree',
    query: treeQueries.detail,
    idParam: 'treeId',
    title: (tree) => getI18n().t('tree:detail.title', { number: tree.number }),
    notFound: {
      entityName: { key: 'tree:entity.name' },
      backTo: '/trees',
      backLabel: { key: 'tree:detail.notFoundBackLabel' },
    },
  }),
)
