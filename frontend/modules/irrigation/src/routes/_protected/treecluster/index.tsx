import { createFileRoute } from '@tanstack/react-router'
import { useQuery, useSuspenseQuery, keepPreviousData } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import { Button, Loading } from '@green-ecolution/ui'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import TreeclusterCard from '../../../components/general/cards/TreeclusterCard'
import ClusterCard from '../../../components/treecluster/ClusterCard'
import Pagination from '@/components/general/Pagination'
import ClusterListToolbar from '../../../components/treecluster/list/ClusterListToolbar'
import {
  DEFAULT_CLUSTER_SORT,
  useClusterListSearch,
} from '../../../components/treecluster/list/useClusterListSearch'
import ClusterViewToggle from '../../../components/treecluster/ClusterViewToggle'
import { z } from 'zod'
import { ListClustersSortEnum, ListClustersOrderEnum } from '@green-ecolution/backend-client'
import { regionsQuery } from '@/api/queries'
import { clusterQueries } from '../../../api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { filterSearchSchema } from '../../../lib/filterSearchSchema'
import { pendingLoading, prefetch } from '@/lib/router'
import { SoilCondition } from '@/api/backendApi'
import { Can } from '@/lib/auth/Can'

export const PER_PAGE = 25

const treeclusterFilterSchema = filterSearchSchema
  .pick({ wateringStatuses: true, regions: true })
  .extend({
    page: z.number().int().min(1).catch(1),
    q: z.string().optional().catch(undefined),
    sort: z.enum(ListClustersSortEnum).optional().catch(undefined),
    order: z.enum(ListClustersOrderEnum).optional().catch(undefined),
    soil: z.array(z.string()).optional().catch(undefined),
    view: z.enum(['cards', 'table']).optional().catch(undefined),
  })

const listParams = (search: z.infer<typeof treeclusterFilterSchema>) => ({
  page: search.page,
  perPage: PER_PAGE,
  wateringStatus: search.wateringStatuses,
  region: search.regions,
  query: search.q,
  sort: search.sort ?? DEFAULT_CLUSTER_SORT,
  order: search.order,
  soilCondition: search.soil as SoilCondition[] | undefined,
})

function Treecluster() {
  const { t } = useTranslation('treecluster')
  const search = Route.useSearch()
  const { view = 'cards' } = search
  const { resetFilters } = useClusterListSearch()
  const {
    data: clustersRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...clusterQueries.list(listParams(search)),
    placeholderData: keepPreviousData,
  })
  const { data: stats } = useSuspenseQuery(clusterQueries.statistics())
  if (error) throw error

  const isFiltered =
    (search.q ?? '').length > 0 ||
    (search.wateringStatuses?.length ?? 0) > 0 ||
    (search.regions?.length ?? 0) > 0 ||
    (search.soil?.length ?? 0) > 0

  return (
    <div className="container mt-6">
      <ListPageHeader
        title={t('list.title')}
        meta={t('list.statsLabel', { total: stats.total, trees: stats.trees })}
        description={t('list.description')}
        action={
          <>
            <ClusterViewToggle />
            <Can permission={['tree_cluster:create']}>
              <ButtonLink
                icon={Plus}
                label={t('list.createButton')}
                link={{ to: '/map/treecluster/new' }}
              />
            </Can>
          </>
        }
      />

      <section>
        <ClusterListToolbar
          filteredRecords={clustersRes?.pagination?.totalRecords ?? 0}
          totalRecords={clustersRes?.pagination?.totalUnfiltered ?? undefined}
        />

        {!clustersRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="mt-6 transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            {clustersRes.data.length === 0 && isFiltered ? (
              <div className="mt-10 text-center">
                <p className="text-dark-600">{t('list.emptyFilteredMessage')}</p>
                <Button variant="outline" className="mt-4" onClick={resetFilters}>
                  {t('list.emptyFilteredAction')}
                </Button>
              </div>
            ) : view === 'table' ? (
              <>
                <ListCardHeader columns="1fr 2fr 1.5fr 1fr">
                  <p>{t('list.columnStatus')}</p>
                  <p>{t('list.columnName')}</p>
                  <p>{t('list.columnLocation')}</p>
                  <p>{t('list.columnTreeCount')}</p>
                </ListCardHeader>

                <EntityList
                  items={clustersRes.data}
                  getKey={(cluster) => cluster.id}
                  emptyMessage={t('list.emptyMessage')}
                  renderItem={(cluster) => <TreeclusterCard treecluster={cluster} />}
                />
              </>
            ) : (
              <EntityList
                layout="grid"
                items={clustersRes.data}
                getKey={(cluster) => cluster.id}
                emptyMessage={t('list.emptyMessage')}
                renderItem={(cluster) => <ClusterCard treecluster={cluster} />}
              />
            )}
            {clustersRes.pagination && clustersRes.pagination?.totalPages > 1 && (
              <Pagination pagination={clustersRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}

export const Route = createFileRoute('/_protected/treecluster/')({
  component: Treecluster,
  validateSearch: treeclusterFilterSchema,
  pendingComponent: pendingLoading({ key: 'treecluster:list.loadingLabel' }),
  loaderDeps: ({ search }) => ({
    page: search.page,
    wateringStatuses: search.wateringStatuses,
    regions: search.regions,
    q: search.q,
    sort: search.sort,
    order: search.order,
    soil: search.soil,
  }),
  loader: ({ deps, context: { queryClient } }) => {
    prefetch(queryClient, clusterQueries.list(listParams(deps)), 'clusterQueries.list')
    prefetch(queryClient, clusterQueries.statistics(), 'clusterQueries.statistics')
    prefetch(queryClient, regionsQuery(), 'regionsQuery')
  },
})
