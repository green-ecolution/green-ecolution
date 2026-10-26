import { planningManifests } from './planning'
import type { I18nKey, ModuleManifest } from './types'

/** The first active module that phrases a core dashboard card wins; without one the core text stays. */
export function cardDescriptionKeys(manifests: readonly ModuleManifest[]): {
  map: I18nKey | undefined
  plans: I18nKey | undefined
} {
  return {
    map: manifests.find(({ map }) => map?.cardDescriptionKey)?.map?.cardDescriptionKey,
    plans: manifests.find(({ planning }) => planning?.cardDescriptionKey)?.planning
      ?.cardDescriptionKey,
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
