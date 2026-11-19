import { useQuery } from '@tanstack/react-query'
import { userQueries } from '@/api/queries'
import { permissionsIn } from './organizationScope'
import { NO_PERMISSIONS, UNRESTRICTED, type Permissions } from './permissions'
import { readAuthBypass } from './runtimeConfig'
import { useEffectiveRoles } from './scope/effectiveRoles'

export type PermissionsResolver = (orgId: string | null) => Permissions

/**
 * The rights per organization, or `undefined` while the roles or the parent
 * chain still load, so a caller can wait instead of reading "no rights".
 * Narrowed by the active organization scope: `useEffectiveRoles` already applies it.
 *
 * The parent chain comes from the reachable organizations rather than the
 * visible ones: a role may sit on an organization the user cannot list.
 */
export function usePermissionsResolver(): PermissionsResolver | undefined {
  const bypass = readAuthBypass()
  const roles = useEffectiveRoles()
  const { data: reachable } = useQuery({ ...userQueries.myOrganizations(), enabled: !bypass })

  if (bypass) return () => UNRESTRICTED
  if (!roles || !reachable) return undefined
  const parentOf = new Map(reachable.map((org) => [org.id, org.parentId]))
  return (orgId: string | null) => (orgId ? permissionsIn(roles, orgId, parentOf) : NO_PERMISSIONS)
}

/** `NO_PERMISSIONS` while loading; see `usePermissionsResolver` to tell the two apart. */
export function usePermissionsIn(orgId: string | null): Permissions {
  return usePermissionsResolver()?.(orgId) ?? NO_PERMISSIONS
}
