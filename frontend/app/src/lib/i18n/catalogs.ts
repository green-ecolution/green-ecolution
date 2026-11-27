import type { UiCatalog } from '@green-ecolution/ui'
import type de from '@/locales/de'

/**
 * Every contributor of translation catalogs, keyed by who ships them. A module
 * adds its own entry by augmenting this interface, so its keys stay checked at
 * compile time without the core naming it. `ui` rides with the core because it
 * ships from the component package, not from a locale barrel.
 */
export interface Catalogs {
  core: typeof de & { ui: UiCatalog }
}

type UnionToIntersection<U> = (U extends unknown ? (member: U) => void : never) extends (
  member: infer I,
) => void
  ? I
  : never

export type CatalogResources = UnionToIntersection<Catalogs[keyof Catalogs]>
