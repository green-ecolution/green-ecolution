import { lazy } from 'react'
import type { ModuleMap } from '@/modules/types'
import { INTERACTIVE_LAYERS } from './lib/mapStyle'
import { wasteMapSearchSchema } from './lib/mapSearch'

// Heavier code loads on demand: the manifest sits in the core bundle next to the registry.
const loadOverview = () => import('./components/map/WasteMapOverview')
const loadOverlay = () => import('./components/map/WasteMapOverlay')

const prefetch: NonNullable<ModuleMap['prefetch']> = (queryClient) => {
  Promise.all([
    import('./api/queries').then(({ wasteBinQueries }) =>
      queryClient.prefetchQuery(wasteBinQueries.markers()),
    ),
    loadOverview(),
    loadOverlay(),
  ]).catch(() => undefined)
}

export const wasteMap: ModuleMap = {
  permissions: ['waste_bin:read'],
  searchSchema: wasteMapSearchSchema,
  interactiveLayerIds: INTERACTIVE_LAYERS,
  prefetch,
  Overview: lazy(loadOverview),
  Overlay: lazy(loadOverlay),
}
