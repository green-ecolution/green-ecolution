import { satisfies, type PermissionRequirement } from './permissions'
import { usePermissions } from './usePermissions'

/** True when the current user satisfies `required` (OR semantics; empty = allowed). */
export function useHasPermission(required: PermissionRequirement): boolean {
  return satisfies(usePermissions(), required)
}
