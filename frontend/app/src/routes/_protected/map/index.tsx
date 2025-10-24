import { createFileRoute, useLoaderData, useNavigate } from '@tanstack/react-router'
import MapButtons from '@/components/map/MapButtons'
import { Tree, TreeCluster } from '@green-ecolution/backend-client'
import { useQuery } from '@tanstack/react-query'
import { treeQuery } from '@/api/queries'
import { useRef } from 'react'
import Dialog from '@/components/general/filter/Dialog'
import StatusFieldset from '@/components/general/filter/fieldsets/StatusFieldset'
import FilterProvider from '@/context/FilterContext'
import { z } from 'zod'
import ClusterFieldset from '@/components/general/filter/fieldsets/ClusterFieldset'
import PlantingYearFieldset from '@/components/general/filter/fieldsets/PlantingYearFieldset'
import useMapInteractions from '@/hooks/useMapInteractions'
import { WithTreesAndClusters } from '@/components/map/marker/WithAllClusterAndTrees'
import WithFilterdTrees from '@/components/map/marker/WithFilterdTrees'

const mapFilterSchema = z.object({
  wateringStatuses: z.array(z.string()).optional(),
  hasCluster: z.boolean().optional(),
  plantingYears: z.array(z.number()).optional(),
  tree: z.number().optional(),
  cluster: z.number().optional(),
})

function MapView() {
  const navigate = useNavigate({ from: '/map' })
  const search = useLoaderData({ from: '/_protected/map/' })
  const { enableDragging, disableDragging } = useMapInteractions()
  const dialogRef = useRef<HTMLDivElement>(null)

  const hasActiveFilter = () => {
    return search.wateringStatuses || search.hasCluster !== undefined || search.plantingYears
      ? true
      : false
  }

  const { data: treesRes } = useQuery({
    enabled:
      search.wateringStatuses !== undefined ||
      search.hasCluster !== undefined ||
      search.plantingYears !== undefined,
    ...treeQuery({
      wateringStatuses: search.wateringStatuses,
      hasCluster: search.hasCluster,
      plantingYears: search.plantingYears,
    }),
  })

  const handleTreeClick = (tree: Tree) => {
    navigate({ to: `/trees/$treeId`, params: { treeId: tree.id.toString() } }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  const handleClusterClick = (cluster: TreeCluster) => {
    navigate({
      to: `/treecluster/$treeclusterId`,
      params: { treeclusterId: cluster.id.toString() },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const handleMapInteractions = (isOpen: boolean) => {
    if (isOpen) {
      disableDragging()
    } else {
      enableDragging()
    }
  }

  return (
    <>
      <div className="absolute top-6 left-4">
        <Dialog
          ref={dialogRef}
          headline="Bäume filtern"
          isOnMap
          fullUrlPath={Route.fullPath}
          onToggleOpen={handleMapInteractions}
        >
          <StatusFieldset />
          <ClusterFieldset />
          <PlantingYearFieldset />
        </Dialog>
      </div>
      <MapButtons />
      {hasActiveFilter() ? (
        <WithFilterdTrees
          onClick={handleTreeClick}
          selectedTrees={search.tree ? [search.tree] : []}
          hasHighlightedTree={search.tree}
          filterdTrees={treesRes?.data ?? []}
        />
      ) : (
        <WithTreesAndClusters
          onClickTree={handleTreeClick}
          onClickCluster={handleClusterClick}
          hasHighlightedTree={search.tree}
          hasHighlightedCluster={search.cluster}
        />
      )}
    </>
  )
}

const MapViewWithProvider = () => {
  const search = useLoaderData({ from: '/_protected/map/' })
  return (
    <FilterProvider
      initialStatus={search.wateringStatuses ?? []}
      initialHasCluster={search.hasCluster ?? undefined}
      initialPlantingYears={search.plantingYears ?? []}
    >
      <MapView />
    </FilterProvider>
  )
}

export const Route = createFileRoute('/_protected/map/')({
  component: MapViewWithProvider,
  validateSearch: mapFilterSchema,

  loaderDeps: ({ search: { wateringStatuses, hasCluster, plantingYears, tree, cluster } }) => ({
    wateringStatuses: wateringStatuses ?? undefined,
    hasCluster: hasCluster ?? undefined,
    plantingYears: plantingYears ?? undefined,
    tree: tree ?? undefined,
    cluster: cluster ?? undefined,
  }),

  loader: ({ deps: { wateringStatuses, hasCluster, plantingYears, tree, cluster } }) => {
    return { wateringStatuses, hasCluster, plantingYears, tree, cluster }
  },
})
