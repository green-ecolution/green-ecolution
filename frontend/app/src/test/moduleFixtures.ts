import type { Catalogs } from '@/lib/i18n/catalogs'
import type { ModuleManifest } from '@/modules/types'

// Globbed here instead of read from the registry, which several tests mock.
const manifests = import.meta.glob<ModuleManifest>('../../../modules/*/src/manifest.tsx', {
  eager: true,
  import: 'default',
})
const germanCatalogs = import.meta.glob<object>('../../../modules/*/src/locales/de/index.ts', {
  eager: true,
  import: 'default',
})

function ofModule<T>(found: Record<string, T>, key: string): T {
  const entry = Object.entries(found).find(([file]) => file.includes(`/modules/${key}/src/`))
  if (!entry) throw new Error(`no module package "${key}" under frontend/modules`)
  return entry[1]
}

export function manifestOf(key: string): ModuleManifest {
  return ofModule(manifests, key)
}

export function germanCatalogOf<K extends Exclude<keyof Catalogs, 'core'>>(key: K): Catalogs[K] {
  return ofModule(germanCatalogs, key) as Catalogs[K]
}
