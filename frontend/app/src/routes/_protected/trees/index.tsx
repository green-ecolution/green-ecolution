import { Loading } from '@green-ecolution/ui'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
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
      <ListPageHeader
        title="Auflistung aller Bäume"
        description={
          <>
            Hier finden Sie eine Übersicht aller Bäume in einer Listenansicht. Die Bäume lassen sich
            allerdings auch auf einer{' '}
            <a
              href="/map"
              className="text-green underline hover:text-green-light focus:text-green-light-50"
            >
              Karte
            </a>
            &nbsp;visualisieren.
          </>
        }
        action={
          <ButtonLink icon={Plus} label="Neuen Baum erstellen" link={{ to: '/map/tree/new' }} />
        }
      />

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
            <EntityList
              items={treesRes.data}
              getKey={(tree) => tree.id}
              emptyMessage="Es wurden leider keine Bäume gefunden."
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
