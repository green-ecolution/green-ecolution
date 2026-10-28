import { createFileRoute } from '@tanstack/react-router'
import { entityRoute } from '@/lib/router'
import { wasteBinQueries } from '../../../../../api/queries'

export const Route = createFileRoute('/_protected/waste-bins/_formular/$wasteBinId')(
  entityRoute({
    key: 'wasteBin',
    query: wasteBinQueries.detail,
    idParam: 'wasteBinId',
    title: (bin) => ({ titleKey: 'waste:detail.title', params: { number: bin.number } }),
    notFound: {
      entityName: { key: 'waste:entity.name' },
      backTo: '/waste-bins',
      backLabel: { key: 'waste:detail.notFoundBackLabel' },
    },
  }),
)
