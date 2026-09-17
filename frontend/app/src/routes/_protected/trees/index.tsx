import { Button, Loading } from '@green-ecolution/ui'
import { ListTreesOrderEnum, ListTreesSortEnum } from '@green-ecolution/backend-client'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import TreeCard from '@/components/general/cards/TreeCard'
import { z } from 'zod'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import Pagination from '@/components/general/Pagination'
import TreeListToolbar from '@/components/tree/list/TreeListToolbar'
import { useTreeListSearch } from '@/components/tree/list/useTreeListSearch'
import { treeQueries } from '@/api/queries'
import { filterSearchSchema } from '@/lib/filterSearchSchema'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const PER_PAGE = 25

const treeFilterSchema = filterSearchSchema
  .pick({
    wateringStatuses: true,
    hasCluster: true,
    hasSensor: true,
    clusterIds: true,
    plantingYears: true,
  })
  .extend({
    page: z.number().int().min(1).catch(1),
    q: z.string().optional().catch(undefined),
    sort: z.enum(ListTreesSortEnum).optional().catch(undefined),
    order: z.enum(ListTreesOrderEnum).optional().catch(undefined),
  })

const listParams = (search: z.infer<typeof treeFilterSchema>) => ({
  page: search.page,
  perPage: PER_PAGE,
  q: search.q,
  wateringStatus: search.wateringStatuses,
  hasCluster: search.hasCluster,
  hasSensor: search.hasSensor,
  clusterId: search.clusterIds,
  plantingYear: search.plantingYears,
  sort: search.sort,
  order: search.order,
})

function Trees() {
  const { t } = useTranslation('tree')
  const search = Route.useSearch()
  const { resetFilters } = useTreeListSearch()
  const {
    data: treesRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...treeQueries.list(listParams(search)),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  const isFiltered =
    (search.q ?? '').length > 0 ||
    (search.wateringStatuses?.length ?? 0) > 0 ||
    (search.clusterIds?.length ?? 0) > 0 ||
    (search.plantingYears?.length ?? 0) > 0 ||
    search.hasCluster !== undefined ||
    search.hasSensor !== undefined

  return (
    <div className="container mt-6">
      <ListPageHeader
        title={t('list.title')}
        description={
          <>
            {t('list.descriptionIntro')}{' '}
            <a
              href="/map"
              className="text-green underline hover:text-green-light focus:text-green-light-50"
            >
              {t('list.mapLinkLabel')}
            </a>
            &nbsp;{t('list.descriptionOutro')}
          </>
        }
      />

      <section className="mt-10">
        <TreeListToolbar
          filteredRecords={treesRes?.pagination?.totalRecords ?? 0}
          action={
            <Can permission={['tree:create']}>
              <ButtonLink
                icon={Plus}
                label={t('list.createButton')}
                link={{ to: '/map/tree/new' }}
              />
            </Can>
          }
        />

        {!treesRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="mt-6 transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            {treesRes.data.length === 0 && isFiltered ? (
              <div className="mt-10 text-center">
                <p className="text-dark-600">{t('list.emptyFilteredMessage')}</p>
                <Button variant="outline" className="mt-4" onClick={resetFilters}>
                  {t('list.emptyFilteredAction')}
                </Button>
              </div>
            ) : (
              <EntityList
                items={treesRes.data}
                getKey={(tree) => tree.id}
                emptyMessage={t('list.emptyMessage')}
                renderItem={(tree) => <TreeCard tree={tree} query={search.q ?? ''} />}
              />
            )}
            {treesRes.pagination && treesRes.pagination.totalPages > 1 && (
              <Pagination pagination={treesRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}

export const Route = createFileRoute('/_protected/trees/')({
  component: Trees,
  validateSearch: treeFilterSchema,
  pendingComponent: pendingLoading({ key: 'tree:list.loadingLabel' }),
  loaderDeps: ({ search }) => ({
    page: search.page,
    q: search.q,
    wateringStatuses: search.wateringStatuses,
    hasCluster: search.hasCluster,
    hasSensor: search.hasSensor,
    clusterIds: search.clusterIds,
    plantingYears: search.plantingYears,
    sort: search.sort,
    order: search.order,
  }),
  loader: ({ deps, context: { queryClient } }) => {
    prefetch(queryClient, treeQueries.list(listParams(deps)), 'treeQueries.list')
  },
})
