import { planningManifests } from './planning'
import type { I18nKey, ModuleManifest } from './types'

/**
 * A module words a core dashboard card only while it is the only active module
 * drawing on the map, or the only one planning. With several the card stands
 * for all of them and keeps the core text.
 */
export function cardDescriptionKeys(manifests: readonly ModuleManifest[]): {
  map: I18nKey | undefined
  plans: I18nKey | undefined
} {
  const maps = manifests.flatMap(({ map }) => (map ? [map] : []))
  const planners = planningManifests(manifests)
  return {
    map: maps.length === 1 ? maps[0].cardDescriptionKey : undefined,
    plans: planners.length === 1 ? planners[0].planning.cardDescriptionKey : undefined,
  }
}

/** Like the navigation: without a module drawing on the map or planning, these cards lead nowhere. */
export function coreCardsShown(manifests: readonly ModuleManifest[]): {
  map: boolean
  plans: boolean
} {
  return {
    map: manifests.some(({ map }) => !!map),
    plans: planningManifests(manifests).length > 0,
  }
}
