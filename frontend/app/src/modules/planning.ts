import type { StopOutcomeDto } from '@green-ecolution/backend-client'
import type { ModuleManifest, ModulePlanning } from './types'

export type PlanningManifest = ModuleManifest & { planning: ModulePlanning }

export function planningManifests(manifests: readonly ModuleManifest[]): PlanningManifest[] {
  return manifests.filter((manifest): manifest is PlanningManifest => !!manifest.planning)
}

/** The module the board and the plan form work in: the one in the search, else the first active one. */
export function planModule(
  search: { module?: string },
  manifests: readonly ModuleManifest[],
): PlanningManifest | undefined {
  const planning = planningManifests(manifests)
  return planning.find(({ key }) => key === search.module) ?? planning[0]
}

export function planningFor(
  manifests: readonly ModuleManifest[],
  moduleKey: string,
): ModulePlanning | undefined {
  return planningManifests(manifests).find(({ key }) => key === moduleKey)?.planning
}

/**
 * A plan finishes as a whole: stops the reader cannot see, and so no outcome
 * input can show, count as done with an unknown amount.
 */
export function completeOutcomes(stops: string[], outcomes: StopOutcomeDto[]): StopOutcomeDto[] {
  const reported = new Set(outcomes.map(({ stop }) => stop))
  return [
    ...outcomes,
    ...stops
      .filter((stop) => !reported.has(stop))
      .map((stop) => ({ stop, completed: true, amount: null })),
  ]
}

/**
 * Stops a picker could not show, because they belong to an organization the
 * reader cannot see, survive whatever the picker reports as the new selection.
 */
export function keepUnresolvedStops(
  selected: string[],
  previous: string[],
  shown: string[],
): string[] {
  const visible = new Set(shown)
  return [...selected, ...previous.filter((id) => !visible.has(id) && !selected.includes(id))]
}
