import { createFileRoute } from '@tanstack/react-router'
import { useQuery, useSuspenseQuery, keepPreviousData } from '@tanstack/react-query'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import { Loading } from '@green-ecolution/ui'
import FilterProvider from '@/context/FilterContext'
import TreeClusterList from '@/components/treecluster/TreeClusterList'
import ClusterCardGrid from '@/components/treecluster/ClusterCardGrid'
import Pagination from '@/components/general/Pagination'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import RegionFieldset from '@/components/general/filter/fieldsets/RegionFieldset'
import SoilFieldset from '@/components/general/filter/fieldsets/SoilFieldset'
import ClusterToolbar from '@/components/treecluster/ClusterToolbar'
import ClusterStatusChips from '@/components/treecluster/ClusterStatusChips'
import ClusterViewToggle from '@/components/treecluster/ClusterViewToggle'
import { z } from 'zod'
import { treeClusterQuery, clusterStatisticsQuery } from '@/api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { filterSearchSchema } from '@/lib/filterSearchSchema'
import { SoilCondition } from '@/api/backendApi'

const treeclusterFilterSchema = filterSearchSchema
  .pick({ wateringStatuses: true, regions: true })
  .extend({
    page: z.number().catch(1),
    q: z.string().optional().catch(undefined),
    sort: z.enum(['name', 'moisture', 'trees']).optional().catch(undefined),
    order: z.enum(['asc', 'desc']).optional().catch(undefined),
    soil: z.array(z.string()).optional().catch(undefined),
    view: z.enum(['cards', 'table']).optional().catch(undefined),
  })

function Treecluster() {
  const {
    page,
    wateringStatuses,
    regions,
    q,
    sort = 'name',
    order = 'asc',
    soil,
    view = 'cards',
  } = Route.useSearch()
  const { data: clustersRes, isPlaceholderData } = useQuery({
    ...treeClusterQuery({
      page,
      perPage: 12,
      wateringStatus: wateringStatuses,
      region: regions,
      query: q,
      sort,
      order,
      soilCondition: soil as SoilCondition[] | undefined,
    }),
    placeholderData: keepPreviousData,
  })
  const { data: stats } = useSuspenseQuery(clusterStatisticsQuery())

  return (
    <div className="container mt-6">
      <header className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <article className="max-w-3xl">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl xl:text-5xl">
            Bewässerungsgruppen
          </h1>
          <p className="mt-2 font-lato font-semibold text-dark-700">
            {stats.total} Gruppen · {stats.trees} Bäume
          </p>
          <p className="mt-3 text-sm text-dark-600">
            Hier finden Sie eine Übersicht aller Bewässerungsgruppen. Eine Bewässerungsgruppe
            besteht aus mehreren Bäumen, welche aufgrund ihrer Nähe und Standortbedinungen in einer
            Gruppe zusammengefasst wurden. Die Ausstattung einzelner Bäume mit Sensoren erlaubt eine
            Gesamtaussage über den Bewässerungszustand der vollständigen Gruppe.
          </p>
        </article>
        <div className="flex shrink-0 items-center gap-3">
          <ClusterViewToggle />
          <ButtonLink icon={Plus} label="Gruppe anlegen" link={{ to: '/treecluster/new' }} />
        </div>
      </header>

      <section className="mt-8">
        <div className="mb-6 flex flex-col gap-3 lg:mb-8">
          <div className="flex flex-wrap items-center gap-3">
            <ClusterToolbar />
            <Dialog headline="Bewässerungsgruppen filtern" fullUrlPath={Route.fullPath}>
              <StatusFieldset />
              <RegionFieldset />
              <SoilFieldset />
            </Dialog>
          </div>
          <ClusterStatusChips />
        </div>

        {!clustersRes ? (
          <Loading className="mt-10 justify-center" label="Daten werden geladen" />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            {view === 'table' ? (
              <>
                <ListCardHeader columns="1fr 2fr 1.5fr 1fr">
                  <p>Status</p>
                  <p>Name</p>
                  <p>Standort</p>
                  <p>Anzahl d. Bäume</p>
                </ListCardHeader>

                <TreeClusterList data={clustersRes.data} />
              </>
            ) : (
              <ClusterCardGrid data={clustersRes.data} />
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

const TreeclusterWithProvider = () => (
  <FilterProvider>
    <Treecluster />
  </FilterProvider>
)

export const Route = createFileRoute('/_protected/treecluster/')({
  component: TreeclusterWithProvider,
  validateSearch: treeclusterFilterSchema,
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Daten werden geladen" />,
  loaderDeps: ({ search }) => ({
    page: search.page,
    wateringStatuses: search.wateringStatuses,
    regions: search.regions,
    q: search.q,
    sort: search.sort,
    order: search.order,
    soil: search.soil,
  }),
  loader: ({
    context: { queryClient },
    deps: { page, wateringStatuses, regions, q, sort = 'name', order = 'asc', soil },
  }) => {
    queryClient
      .prefetchQuery(
        treeClusterQuery({
          page,
          perPage: 12,
          wateringStatus: wateringStatuses,
          region: regions,
          query: q,
          sort,
          order,
          soilCondition: soil as SoilCondition[] | undefined,
        }),
      )
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))
    queryClient
      .prefetchQuery(clusterStatisticsQuery())
      .catch((error) => console.error('Prefetching "clusterStatisticsQuery" failed:', error))
  },
})

export default TreeclusterWithProvider
