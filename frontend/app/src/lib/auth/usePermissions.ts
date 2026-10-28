import { readAuthBypass } from './runtimeConfig'
import { useEffectiveRoles } from './scope/effectiveRoles'
import { permissionsOf, UNRESTRICTED, type Permissions } from './permissions'

/**
 * Effective permissions of the logged-in user under the selected organization
 * scope. With auth bypassed the backend grants unrestricted access, so the
 * frontend must not hide anything.
 */
export function usePermissions(): Permissions {
  const roles = useEffectiveRoles()
  return readAuthBypass() ? UNRESTRICTED : permissionsOf(roles)
}
