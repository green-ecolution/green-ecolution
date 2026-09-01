import type de from '@/locales/de'

/** Keys under `navigation:crumb.*`, the only titles a route loader may name. */
export type NavigationCrumbKey = keyof (typeof de)['navigation']['crumb']
