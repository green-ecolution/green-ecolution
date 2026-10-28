import { lazy } from 'react'
import { Trash2 } from 'lucide-react'
import type { ModulePlanning } from '@/modules/types'
import { wasteRoots } from './api/roots'

// Lazy because the manifest sits in the core bundle next to the registry, and
// these components reach back into core modules that import that registry.
const loadSuggestionsColumn = () => import('./components/planning/SuggestionsColumn')

const prefetch: NonNullable<ModulePlanning['prefetch']> = (queryClient) => {
  // A failed chunk surfaces when the lazy component renders.
  Promise.all([
    import('./api/queries').then(({ wasteBinQueries }) =>
      queryClient.prefetchQuery(wasteBinQueries.markers()),
    ),
    loadSuggestionsColumn(),
  ]).catch(() => undefined)
}

export const wastePlanning: ModulePlanning = {
  planResource: 'collection_plan',
  plansLabelKey: 'waste:planning.plansLabel',
  texts: {
    listDescription: 'waste:planning.texts.listDescription',
    listDescriptionCanModify: 'waste:planning.texts.listDescriptionCanModify',
    createDescription: 'waste:planning.texts.createDescription',
    updateDescription: 'waste:planning.texts.updateDescription',
    statusUpdateDescription: 'waste:planning.texts.statusUpdateDescription',
    routeErrorAlert: 'waste:planning.texts.routeErrorAlert',
    routePreviewAriaLabel: 'waste:planning.texts.routePreviewAriaLabel',
  },
  capacityKind: 'cargo_volume',
  labelKey: 'waste:planning.label',
  stopsLabelKey: 'waste:planning.stopsLabel',
  stopCountLabelKey: 'waste:planning.stopCountLabel',
  stopCountKey: 'waste:planning.stopCount',
  stopIcon: Trash2,
  // Finishing a tour marks its completed bins as emptied.
  invalidateOnStatusChange: wasteRoots.bin,
  // The picker is a dialog on the plan form and never leaves the page.
  stopPickerPaths: [],
  prefetch,
  SuggestionsColumn: lazy(loadSuggestionsColumn),
  StopList: lazy(() => import('./components/planning/BinStopList')),
  PlanSummary: lazy(() => import('./components/planning/BinPlanSummary')),
  StopPicker: lazy(() => import('./components/planning/BinStopPicker')),
  RouteOverlay: lazy(() => import('./components/planning/BinRouteOverlay')),
  OutcomeInput: lazy(() => import('./components/planning/BinOutcomeInput')),
}
