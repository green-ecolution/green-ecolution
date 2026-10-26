import type { TargetRefDto } from '@green-ecolution/backend-client'
import type { ModuleManifest, ModuleSensorListFilter, ModuleTarget } from '@/modules/types'

export interface SensorTargetKind {
  module: string
  target: ModuleTarget
}

export interface SensorListFilterEntry {
  module: string
  kind: string
  filter: ModuleSensorListFilter
}

export function sensorTargetKinds(manifests: readonly ModuleManifest[]): SensorTargetKind[] {
  return manifests.flatMap(({ key, targets }) =>
    (targets ?? []).map((target) => ({ module: key, target })),
  )
}

/** Undefined for a target whose module is unknown or not among the given manifests. */
export function targetDescriptor(
  manifests: readonly ModuleManifest[],
  ref: TargetRefDto | null | undefined,
): ModuleTarget | undefined {
  if (!ref) return undefined
  return sensorTargetKinds(manifests).find(
    ({ module, target }) => module === ref.module && target.kind === ref.kind,
  )?.target
}

export function sensorListFiltersOf(manifests: readonly ModuleManifest[]): SensorListFilterEntry[] {
  return sensorTargetKinds(manifests).flatMap(({ module, target }) =>
    target.listFilter ? [{ module, kind: target.kind, filter: target.listFilter }] : [],
  )
}

/** A module's search param is untyped in the core schema; anything but a string list reads as unset. */
export function listFilterValue(search: Record<string, unknown>, param: string): string[] {
  const value = search[param]
  return Array.isArray(value)
    ? value.filter((entry): entry is string => typeof entry === 'string')
    : []
}
