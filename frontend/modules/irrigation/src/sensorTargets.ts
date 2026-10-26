import { lazy } from 'react'
import { queryOptions } from '@tanstack/react-query'
import type { ModuleTarget } from '@/modules/types'
import { irrigationRoots } from './api/roots'
import { TREE_KIND } from './lib/targets'

// Lazy for the same reason as the map and planning slots: the manifest sits in
// the core bundle next to the registry, and these components reach back into
// core modules that import that registry. The API clients load on demand too,
// so that importing the registry does not pin them before a test mocks them.
const loadClusterFilter = () => import('./components/sensor/list/ClusterSensorFilter')

/** Sensors of the given clusters; under the core's `['sensors']` key so sensor invalidation reaches it. */
const sensorIdsOfClusters = (clusterIds: string[]) => {
  const sortedIds = clusterIds.slice().sort()
  return queryOptions<string[]>({
    queryKey: ['sensors', 'by-cluster', sortedIds],
    queryFn: () =>
      import('./api/queries').then(({ fetchSensorIdsOfClusters }) =>
        fetchSensorIdsOfClusters(sortedIds),
      ),
  })
}

export const treeSensorTarget: ModuleTarget = {
  kind: TREE_KIND,
  labelKey: 'irrigation:sensor.target.tree',
  texts: {
    activateAction: 'irrigation:sensor.treeTexts.activateAction',
    reassignAction: 'irrigation:sensor.treeTexts.reassignAction',
    removeLinkAction: 'irrigation:sensor.treeTexts.removeLinkAction',
    removeConfirmTitle: 'irrigation:sensor.treeTexts.removeConfirmTitle',
    removeConfirmDescription: 'irrigation:sensor.treeTexts.removeConfirmDescription',
    activateSuccess: 'irrigation:sensor.treeTexts.activateSuccess',
    reassignSuccess: 'irrigation:sensor.treeTexts.reassignSuccess',
    activateConflict: 'irrigation:sensor.treeTexts.activateConflict',
    reassignNotFound: 'irrigation:sensor.treeTexts.reassignNotFound',
    reassignConflict: 'irrigation:sensor.treeTexts.reassignConflict',
    reassignFailed: 'irrigation:sensor.treeTexts.reassignFailed',
    scanDescription: 'irrigation:sensor.treeTexts.scanDescription',
    alreadyLinkedDescription: 'irrigation:sensor.treeTexts.alreadyLinkedDescription',
    listSearchPlaceholder: 'irrigation:sensor.treeTexts.listSearchPlaceholder',
  },
  invalidate: irrigationRoots.tree,
  AssignDialog: lazy(() => import('./components/sensor/SensorTreeAssignDialog')),
  WizardStep: lazy(() => import('./components/sensor/wizard/SensorTreeStep')),
  Placement: lazy(() => import('./components/sensor/TreePlacement')),
  listFilter: {
    // The sensor list linked clusters as `clusterIds` before the filter moved here.
    param: 'clusterIds',
    Control: lazy(() =>
      loadClusterFilter().then((m) => ({ default: m.ClusterSensorFilterControl })),
    ),
    Chips: lazy(() => loadClusterFilter().then((m) => ({ default: m.ClusterSensorFilterChips }))),
    sensorIds: sensorIdsOfClusters,
  },
}
