import { lazy } from 'react'
import { FolderClosed } from 'lucide-react'
import type { ModulePlanning } from '@/modules/types'
import { irrigationRoots } from './api/roots'

// Lazy for the same reason as the map slots: the manifest sits in the core
// bundle next to the registry, and these components reach back into core
// modules that import that registry.
const loadSuggestionsColumn = () => import('./components/planning/SuggestionsColumn')

const prefetch: NonNullable<ModulePlanning['prefetch']> = (queryClient) => {
  // A failed chunk surfaces when the lazy component renders.
  Promise.all([
    import('./api/queries').then(({ clusterQueries }) =>
      queryClient.prefetchQuery(clusterQueries.suggested()),
    ),
    loadSuggestionsColumn(),
  ]).catch(() => undefined)
}

export const irrigationPlanning: ModulePlanning = {
  planResource: 'watering_plan',
  cardDescriptionKey: 'irrigation:dashboard.plansCardDescription',
  plansLabelKey: 'irrigation:planning.plansLabel',
  texts: {
    listDescription: 'irrigation:planning.texts.listDescription',
    listDescriptionCanModify: 'irrigation:planning.texts.listDescriptionCanModify',
    createDescription: 'irrigation:planning.texts.createDescription',
    updateDescription: 'irrigation:planning.texts.updateDescription',
    statusUpdateDescription: 'irrigation:planning.texts.statusUpdateDescription',
    routeErrorAlert: 'irrigation:planning.texts.routeErrorAlert',
    routePreviewAriaLabel: 'irrigation:planning.texts.routePreviewAriaLabel',
  },
  capacityKind: 'water',
  labelKey: 'irrigation:label',
  stopsLabelKey: 'irrigation:planning.stopsLabel',
  stopCountLabelKey: 'irrigation:planning.stopCountLabel',
  stopCountKey: 'irrigation:planning.stopCount',
  stopIcon: FolderClosed,
  // Finishing a plan flags its clusters and their trees as just watered, and
  // the evaluation aggregates finished plans.
  invalidateOnStatusChange: [
    ...irrigationRoots.cluster,
    ...irrigationRoots.tree,
    ...irrigationRoots.evaluation,
  ],
  stopPickerPaths: ['/map/watering-plan/select/cluster'],
  prefetch,
  SuggestionsColumn: lazy(loadSuggestionsColumn),
  StopList: lazy(() => import('./components/planning/ClusterStopList')),
  PlanSummary: lazy(() => import('./components/planning/ClusterPlanSummary')),
  StopPicker: lazy(() => import('./components/planning/ClusterStopPicker')),
  RouteOverlay: lazy(() => import('./components/planning/ClusterRouteOverlay')),
  OutcomeInput: lazy(() => import('./components/planning/ClusterOutcomeInput')),
}
