import type { ModuleListResponse } from '@green-ecolution/backend-client'
import type { ModuleManifest } from './types'

export function activeModuleKeys(response: ModuleListResponse): Set<string> {
  return new Set(response.organizations.flatMap((entry) => entry.active))
}

export function activeManifests(
  manifests: readonly ModuleManifest[],
  active: ReadonlySet<string>,
): ModuleManifest[] {
  return manifests.filter((manifest) => active.has(manifest.key))
}
