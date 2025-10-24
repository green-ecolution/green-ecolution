import { createFileRoute, useLoaderData } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import LoadingInfo from '@/components/general/error/LoadingInfo'
import FilterProvider from '@/context/FilterContext'
import TreeClusterList from '@/components/treecluster/TreeClusterList'
import Pagination from '@/components/general/Pagination'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import RegionFieldset from '@/components/general/filter/fieldsets/RegionFieldset'
import { GetAllTreeClustersRequest } from '@green-ecolution/backend-client'
import { z } from 'zod'
import { treeClusterQuery } from '@/api/queries'

const treeclusterFilterSchema = z.object({
  wateringStatuses: z.array(z.string()).optional(),
  regions: z.array(z.string()).optional(),
  page: z.number().catch(1),
})

function Treecluster() {
  const { page, wateringStatuses, regions } = useLoaderData({ from: '/_protected/treecluster/' })
  const { data: clustersRes } = useSuspenseQuery(
    treeClusterQuery({
      page: page,
      limit: 5,
      wateringStatuses,
      regions,
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
          </Dialog>
        </div>

        <header className="hidden border-b pb-2 text-sm text-dark-800 px-8 border-b-dark-200 mb-5 lg:grid lg:grid-cols-[1fr,2fr,1.5fr,1fr] lg:gap-5 xl:px-10">
          <p>Status</p>
          <p>Name</p>
          <p>Standort</p>
          <p>Anzahl d. Bäume</p>
        </header>

        <TreeClusterList data={clustersRes.data} />
        {clustersRes.pagination && clustersRes.pagination?.totalPages > 1 && (
          <Pagination pagination={clustersRes.pagination} />
        )}
      </section>
    </div>
  )
}

const TreeclusterWithProvider = () => {
  const search = useLoaderData({ from: '/_protected/treecluster/' })

  return (
    <FilterProvider
      initialStatus={search.wateringStatuses ?? []}
      initialRegions={search.regions ?? []}
    >
      <Treecluster />
    </FilterProvider>
  )
}

export const Route = createFileRoute('/_protected/treecluster/')({
  component: TreeclusterWithProvider,
  validateSearch: treeclusterFilterSchema,
  pendingComponent: () => <LoadingInfo label="Daten werden geladen" />,
  loaderDeps: ({ search }: { search: GetAllTreeClustersRequest }) => ({
    wateringStatuses:
      search.wateringStatuses && search.wateringStatuses.length > 0
        ? search.wateringStatuses
        : undefined,

    regions: search.regions && search.regions.length > 0 ? search.regions : undefined,

    page: search.page,
  }),
  loader: ({ context: { queryClient }, deps: { wateringStatuses, regions, page } }) => {
    queryClient
      .prefetchQuery(
        treeClusterQuery({
          page,
          limit: 5,
          wateringStatuses,
          regions,
        }),
      )
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))

    return { wateringStatuses, regions, page }
  },
})

export default TreeclusterWithProvider
