import LoadingInfo from '@/components/general/error/LoadingInfo'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, useLoaderData } from '@tanstack/react-router'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import TreeCard from '@/components/general/cards/TreeCard'
import { z } from 'zod'
import Pagination from '@/components/general/Pagination'
import { GetAllTreesRequest } from '@green-ecolution/backend-client'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import ClusterFieldset from '@/components/general/filter/fieldsets/ClusterFieldset'
import PlantingYearFieldset from '@/components/general/filter/fieldsets/PlantingYearFieldset'
import FilterProvider from '@/context/FilterContext'
import { treeQuery } from '@/api/queries'

const treeFilterSchema = z.object({
  wateringStatuses: z.array(z.string()).optional(),
  hasCluster: z.boolean().optional(),
  plantingYears: z.array(z.number()).optional(),
})

function Trees() {
  const { wateringStatuses, hasCluster, plantingYears, page } = useLoaderData({
    from: '/_protected/trees/',
  })
  const { data: treesRes } = useSuspenseQuery(
    treeQuery({
      page: page ?? 1,
      wateringStatuses,
      hasCluster,
      plantingYears,
      limit: 10,
    }),
  )

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
        <header className="hidden border-b pb-2 text-sm text-dark-800 px-6 border-b-dark-200 mb-5 lg:grid lg:grid-cols-[1fr,1.5fr,1fr,1fr] lg:gap-5 lg:px-10">
          <p>Status</p>
          <p>Baumart</p>
          <p>Baumnummer</p>
          <p>Bewässerungsgruppe</p>
        </header>
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
      </section>
    </div>
  )
}

const TreesWithProvider = () => {
  const search = useLoaderData({ from: '/_protected/trees/' })

  return (
    <FilterProvider
      initialStatus={search.wateringStatuses}
      initialHasCluster={search.hasCluster}
      initialPlantingYears={search.plantingYears}
    >
      <Trees />
    </FilterProvider>
  )
}

export const Route = createFileRoute('/_protected/trees/')({
  component: TreesWithProvider,
  validateSearch: treeFilterSchema,
  pendingComponent: () => <LoadingInfo label="Daten werden geladen" />,
  loaderDeps: ({ search }: { search: GetAllTreesRequest }) => ({
    wateringStatuses: search.wateringStatuses ?? undefined,
    hasCluster: search.hasCluster ?? undefined,
    plantingYears: search.plantingYears ?? undefined,
    page: search.page ?? 1,
  }),
  loader: ({
    deps: { page, wateringStatuses, hasCluster, plantingYears },
    context: { queryClient },
  }) => {
    const query = queryClient.prefetchQuery(
      treeQuery({
        page,
        wateringStatuses,
        hasCluster,
        plantingYears,
        limit: 10,
      }),
    )

    return { page, wateringStatuses, hasCluster, plantingYears, query }
  },
})
