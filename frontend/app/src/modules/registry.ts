import type { ModuleManifest } from './types'

const found = import.meta.glob<ModuleManifest>('../../../modules/*/src/manifest.tsx', {
  eager: true,
  import: 'default',
})

export function sortManifests(manifests: readonly ModuleManifest[]): ModuleManifest[] {
  return [...manifests].sort((a, b) => a.order - b.order || a.key.localeCompare(b.key))
}

export const moduleManifests: readonly ModuleManifest[] = sortManifests(Object.values(found))
