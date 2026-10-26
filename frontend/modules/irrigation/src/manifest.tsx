import type { ModuleManifest } from '@/modules/types'

export const irrigationManifest: ModuleManifest = {
  key: 'irrigation',
  labelKey: 'common:modules.irrigation',
  i18n: {
    namespaces: ['tree', 'treecluster', 'evaluation', 'irrigation'],
    // One `import()` per language literal, so each language gets its own chunk.
    load: async (lng) =>
      lng === 'en'
        ? (await import('./locales/en')).default
        : (await import('./locales/de')).default,
  },
}
