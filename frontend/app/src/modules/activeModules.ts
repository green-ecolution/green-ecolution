import type { ModuleListResponse } from '@green-ecolution/backend-client'
import type { ModuleManifest } from './types'

export function activeModuleKeys(response: ModuleListResponse): Set<string> {
  return new Set(response.organizations.flatMap((entry) => entry.active))
}

export function activeManifestsFrom(
  manifests: readonly ModuleManifest[],
  response: ModuleListResponse,
): ModuleManifest[] {
  const registered = new Set(response.modules)
  const active = activeModuleKeys(response)
  return manifests.filter(({ key }) => registered.has(key) && active.has(key))
}

export function activeManifestsIn(
  manifests: readonly ModuleManifest[],
  response: ModuleListResponse,
  organizationId: string,
): ModuleManifest[] {
  const registered = new Set(response.modules)
  const active = new Set(
    response.organizations.find((entry) => entry.organizationId === organizationId)?.active ?? [],
  )
  return manifests.filter(({ key }) => registered.has(key) && active.has(key))
}
