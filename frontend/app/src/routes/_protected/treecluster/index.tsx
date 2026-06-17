import { createFileRoute } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
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
import { z } from 'zod'
import { treeClusterQuery } from '@/api/queries'
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
  const { data: clustersRes } = useSuspenseQuery(
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

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Auflistung der Bewässerungsgruppen
        </h1>
        <p className="mb-5">
          Hier finden Sie eine Übersicht aller Bewässerungsgruppen. Eine Bewässerungsgruppe besteht
          aus mehreren Bäumen, welche aufgrund ihrer Nähe und Standortbedinungen in einer Gruppe
          zusammengefasst wurden. Die Ausstattung einzelner Bäume mit Sensoren erlaubt eine
          Gesamtaussage über den Bewässerungszustand der vollständigen Gruppe. Die Auswertung der
          Daten aller Sensoren einer Bewässerungsgruppe liefert Handlungsempfehlungen für diese
          Gruppe.
        </p>
        <ButtonLink icon={Plus} label="Neue Gruppe erstellen" link={{ to: '/treecluster/new' }} />
      </article>

      <section className="mt-10">
        <div className="flex justify-end mb-6 lg:mb-10">
          <Dialog headline="Bewässerungsgruppen filtern" fullUrlPath={Route.fullPath}>
            <StatusFieldset />
            <RegionFieldset />
            <SoilFieldset />
          </Dialog>
        </div>

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
  },
})

export default TreeclusterWithProvider
