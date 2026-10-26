import { lazy } from 'react'
import type { ModuleMap } from '@/modules/types'
import { INTERACTIVE_LAYERS } from './lib/mapStyle'
import { irrigationMapSearchSchema } from './lib/mapSearch'

// Everything heavier than constants loads on demand: the manifest sits in the
// core bundle next to the registry, and the map components and the API client
// reach back into core modules that import that registry.
const loadOverview = () => import('./components/map/IrrigationMapOverview')
const loadToolbar = () => import('./components/map/MapToolbarBar')
const loadOverlay = () => import('./components/map/IrrigationMapOverlay')

const prefetch: NonNullable<ModuleMap['prefetch']> = (queryClient) => {
  // A failed chunk surfaces when the lazy component renders.
  Promise.all([
    import('./api/queries').then(({ clusterQueries }) =>
      Promise.all([
        queryClient.prefetchQuery(clusterQueries.markers()),
        queryClient.prefetchQuery(clusterQueries.boundaries()),
      ]),
    ),
    loadOverview(),
    loadToolbar(),
    loadOverlay(),
  ]).catch(() => undefined)
}

export const irrigationMap: ModuleMap = {
  permissions: ['tree:read', 'tree_cluster:read'],
  searchSchema: irrigationMapSearchSchema,
  interactiveLayerIds: INTERACTIVE_LAYERS,
  prefetch,
  Overview: lazy(loadOverview),
  Toolbar: lazy(loadToolbar),
  Overlay: lazy(loadOverlay),
}
