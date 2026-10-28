import { Trash2 } from 'lucide-react'
import type { ModuleManifest } from '@/modules/types'
import { wasteMap } from './map'
import { wastePlanning } from './planning'
import { WASTE } from './lib/targets'
import { wasteResources } from './resources'

export const wasteManifest: ModuleManifest = {
  key: WASTE,
  labelKey: 'waste:label',
  i18n: {
    namespaces: ['waste'],
    // One `import()` per language literal, so each language gets its own chunk.
    load: async (lng) =>
      lng === 'en'
        ? (await import('./locales/en')).default
        : (await import('./locales/de')).default,
  },
  nav: {
    links: [
      {
        key: 'nav-assets-waste-bins',
        to: '/waste-bins',
        labelKey: 'waste:nav.bins',
        icon: Trash2,
        section: 'assets',
      },
    ],
  },
  routePermissions: {
    '/waste-bins': ['waste_bin:read'],
  },
  resources: wasteResources,
  map: wasteMap,
  planning: wastePlanning,
  statistics: {
    bin: {
      icon: Trash2,
      labelKey: 'waste:statistics.bin.label',
      to: '/waste-bins',
    },
  },
}
