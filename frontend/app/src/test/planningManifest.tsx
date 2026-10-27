import { MapPin } from 'lucide-react'
import type { ModuleManifest, ModulePlanning } from '@/modules/types'

const slot = (name: string) => {
  const Slot = () => <div data-testid={name} />
  Slot.displayName = name
  return Slot
}

/** A planning module whose slots render only a test id, so core tests see which slot it uses. */
export function probePlanning(overrides: Partial<ModulePlanning> = {}): ModulePlanning {
  return {
    planResource: 'watering_plan',
    capacityKind: 'water',
    labelKey: 'irrigation:label',
    stopsLabelKey: 'irrigation:planning.stopsLabel',
    stopCountLabelKey: 'irrigation:planning.stopCountLabel',
    stopCountKey: 'irrigation:planning.stopCount',
    stopIcon: MapPin,
    invalidateOnStatusChange: [],
    stopPickerPaths: [],
    SuggestionsColumn: slot('suggestions'),
    StopList: slot('stop-list'),
    PlanSummary: slot('plan-summary'),
    StopPicker: slot('stop-picker'),
    RouteOverlay: slot('route-overlay'),
    OutcomeInput: slot('outcome-input'),
    ...overrides,
  }
}

export function probeManifest(
  key: string,
  planning: Partial<ModulePlanning> = {},
): ModuleManifest & { planning: ModulePlanning } {
  return {
    key,
    labelKey: 'irrigation:label',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    planning: probePlanning({
      SuggestionsColumn: slot(`suggestions-${key}`),
      StopList: slot(`stop-list-${key}`),
      PlanSummary: slot(`plan-summary-${key}`),
      StopPicker: slot(`stop-picker-${key}`),
      RouteOverlay: slot(`route-overlay-${key}`),
      OutcomeInput: slot(`outcome-input-${key}`),
      ...planning,
    }),
  }
}
