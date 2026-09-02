import { Loading } from '@green-ecolution/ui'
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
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import ClusterFieldset from '@/components/general/filter/fieldsets/ClusterFieldset'
import PlantingYearFieldset from '@/components/general/filter/fieldsets/PlantingYearFieldset'
import { treeQueries } from '@/api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { filterSearchSchema } from '@/lib/filterSearchSchema'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

const treeFilterSchema = filterSearchSchema
  .pick({ wateringStatuses: true, hasCluster: true, plantingYears: true })
  .extend({ page: z.number().int().min(1).catch(1) })

function Trees() {
  const { t } = useTranslation('tree')
  const { page, wateringStatuses, hasCluster, plantingYears } = Route.useSearch()
  const {
    data: treesRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...treeQueries.list({
      page,
      perPage: 10,
      wateringStatus: wateringStatuses,
      hasCluster,
      plantingYear: plantingYears,
    }),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

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
        action={
          <Can permission={['tree:create']}>
            <ButtonLink icon={Plus} label={t('list.createButton')} link={{ to: '/map/tree/new' }} />
          </Can>
        }
      />

      <section className="mt-10">
        <div className="flex justify-end mb-6 lg:mb-10">
          <Dialog headline={t('list.filterHeadline')} fullUrlPath={Route.fullPath}>
            <StatusFieldset />
            <ClusterFieldset />
            <PlantingYearFieldset />
          </Dialog>
        </div>
        <ListCardHeader columns="1fr 1.5fr 1fr 1fr">
          <p>{t('list.columnStatus')}</p>
          <p>{t('list.columnSpecies')}</p>
          <p>{t('list.columnNumber')}</p>
          <p>{t('list.columnCluster')}</p>
        </ListCardHeader>
        {!treesRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <EntityList
              items={treesRes.data}
              getKey={(tree) => tree.id}
              emptyMessage={t('list.emptyMessage')}
              renderItem={(tree) => <TreeCard tree={tree} />}
            />
            {treesRes.pagination && treesRes.pagination?.totalPages > 1 && (
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
    wateringStatuses: search.wateringStatuses,
    hasCluster: search.hasCluster,
    plantingYears: search.plantingYears,
  }),
  loader: ({
    deps: { page, wateringStatuses, hasCluster, plantingYears },
    context: { queryClient },
  }) => {
    prefetch(
      queryClient,
      treeQueries.list({
        page,
        perPage: 10,
        wateringStatus: wateringStatuses,
        hasCluster,
        plantingYear: plantingYears,
      }),
      'treeQueries.list',
    )
  },
})
