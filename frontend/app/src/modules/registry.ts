import type { ModuleManifest } from './types'

const found = import.meta.glob<ModuleManifest>('../../../modules/*/src/manifest.tsx', {
  eager: true,
  import: 'default',
})

export function sortManifests(manifests: readonly ModuleManifest[]): ModuleManifest[] {
  // By code point: `localeCompare` would order the same keys differently per browser locale.
  return [...manifests].sort(
    (a, b) => a.order - b.order || (a.key < b.key ? -1 : a.key > b.key ? 1 : 0),
  )
}

export const moduleManifests: readonly ModuleManifest[] = sortManifests(Object.values(found))
