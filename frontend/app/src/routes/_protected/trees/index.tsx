import { Loading } from '@green-ecolution/ui'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import TreeCard from '@/components/general/cards/TreeCard'
import { z } from 'zod'
import Pagination from '@/components/general/Pagination'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import ClusterFieldset from '@/components/general/filter/fieldsets/ClusterFieldset'
import PlantingYearFieldset from '@/components/general/filter/fieldsets/PlantingYearFieldset'
import FilterProvider from '@/context/FilterContext'
import { treeQuery } from '@/api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { filterSearchSchema } from '@/lib/filterSearchSchema'
import { pendingLoading, prefetch } from '@/lib/router'

const treeFilterSchema = filterSearchSchema
  .pick({ wateringStatuses: true, hasCluster: true, plantingYears: true })
  .extend({ page: z.number().int().min(1).catch(1) })

function Trees() {
  const { page, wateringStatuses, hasCluster, plantingYears } = Route.useSearch()
  const {
    data: treesRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...treeQuery({
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
      <article className="mb-20 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Auflistung aller Bäume
        </h1>
        <p className="mb-5">
          Hier finden Sie eine Übersicht aller Bäume in einer Listenansicht. Die Bäume lassen sich
          allerdings auch auf einer{' '}
          <a
            href="/map"
            className="text-green underline hover:text-green-light focus:text-green-light-50"
          >
            Karte
          </a>
          &nbsp;visualisieren.
        </p>
        <ButtonLink icon={Plus} label="Neuen Baum erstellen" link={{ to: '/map/tree/new' }} />
      </article>

      <section className="mt-10">
        <div className="flex justify-end mb-6 lg:mb-10">
          <Dialog headline="Bäume filtern" fullUrlPath={Route.fullPath}>
            <StatusFieldset />
            <ClusterFieldset />
            <PlantingYearFieldset />
          </Dialog>
        </div>
        <ListCardHeader columns="1fr 1.5fr 1fr 1fr">
          <p>Status</p>
          <p>Baumart</p>
          <p>Baumnummer</p>
          <p>Bewässerungsgruppe</p>
        </ListCardHeader>
        {!treesRes ? (
          <Loading className="mt-10 justify-center" label="Daten werden geladen" />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <ul>
              {treesRes.data?.length === 0 ? (
                <li className="text-center text-dark-600 mt-10">
                  <p>Es wurden leider keine Bäume gefunden.</p>
                </li>
              ) : (
                treesRes.data?.map((tree) => (
                  <li key={tree.id} className="mb-5 last:mb-0">
                    <TreeCard tree={tree} />
                  </li>
                ))
              )}
            </ul>
            {treesRes.pagination && treesRes.pagination?.totalPages > 1 && (
              <Pagination pagination={treesRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}

const TreesWithProvider = () => (
  <FilterProvider>
    <Trees />
  </FilterProvider>
)

export const Route = createFileRoute('/_protected/trees/')({
  component: TreesWithProvider,
  validateSearch: treeFilterSchema,
  pendingComponent: pendingLoading('Daten werden geladen'),
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
      treeQuery({
        page,
        perPage: 10,
        wateringStatus: wateringStatuses,
        hasCluster,
        plantingYear: plantingYears,
      }),
      'treeQuery',
    )
  },
})
