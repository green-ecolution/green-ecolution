import { useSuspenseQuery } from '@tanstack/react-query'
import type { ModuleListResponse } from '@green-ecolution/backend-client'
import { modulesQuery } from '@/api/queries'
import { useSettingsHomeOrganizationId } from '@/lib/auth/scope/settingsOrganization'
import { activeManifestsFrom, activeManifestsIn } from './activeModules'
import { moduleManifests } from './registry'
import type { ModuleCapacityKind, ModuleManifest } from './types'

/** A stored kind keeps its manifest's wording even where its module is switched off. */
export function capacityKindOf(
  kind: string,
  manifests: readonly ModuleManifest[] = moduleManifests,
): ModuleCapacityKind | undefined {
  return manifests.flatMap((m) => m.capacities ?? []).find((c) => c.kind === kind)
}

/** `organizationId` null: not yet known, so every reachable organization counts. */
export function capacityKindsIn(
  manifests: readonly ModuleManifest[],
  response: ModuleListResponse,
  organizationId: string | null,
): ModuleCapacityKind[] {
  const active = organizationId
    ? activeManifestsIn(manifests, response, organizationId)
    : activeManifestsFrom(manifests, response)
  return active.flatMap((m) => m.capacities ?? [])
}

/**
 * The kinds a vehicle or station of `organizationId` offers. Without one, the
 * organization a new object lands in: the scope, else the user's own.
 */
export function useCapacityKinds(organizationId: string | undefined): ModuleCapacityKind[] {
  const { data } = useSuspenseQuery(modulesQuery())
  const home = useSettingsHomeOrganizationId()
  return capacityKindsIn(moduleManifests, data, organizationId ?? home ?? null)
}
