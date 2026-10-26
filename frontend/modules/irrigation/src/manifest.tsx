import type { ModuleManifest } from '@/modules/types'

export const irrigationManifest: ModuleManifest = {
  key: 'irrigation',
  labelKey: 'common:modules.irrigation',
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
}
