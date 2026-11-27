import { Trash2 } from 'lucide-react'
import type { ModuleManifest } from '@/modules/types'
import { wasteMap } from './map'
import { wastePlanning } from './planning'
import { WASTE } from './lib/targets'
import { wasteResources } from './resources'
import { binSensorTarget } from './sensorTargets'
import { wasteSettingsSection } from './components/settings/settingsSection'

export const wasteManifest: ModuleManifest = {
  key: WASTE,
  order: 20,
  labelKey: 'waste:label',
  icon: Trash2,
  descriptionKey: 'waste:description',
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
  roleTemplates: [
    { key: 'waste_collection', nameKey: 'waste:roles.waste_collection', icon: Trash2 },
  ],
  map: wasteMap,
  planning: wastePlanning,
  targets: [binSensorTarget],
  settingsSection: wasteSettingsSection,
  statistics: {
    bin: {
      icon: Trash2,
      labelKey: 'waste:statistics.bin.label',
      to: '/waste-bins',
    },
  },
}

export default wasteManifest
