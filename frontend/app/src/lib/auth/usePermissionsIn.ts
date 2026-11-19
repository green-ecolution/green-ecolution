import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { userQueries } from '@/api/queries'
import { permissionsIn } from './organizationScope'
import { NO_PERMISSIONS, UNRESTRICTED, type Permissions } from './permissions'
import { readAuthBypass } from './runtimeConfig'
import { useEffectiveRoles } from './scope/effectiveRoles'

/**
 * The parent chain comes from the reachable organizations rather than the
 * visible ones: a role may sit on an organization the user cannot list.
 */
export function usePermissionsIn(orgId: string | null): Permissions {
  const bypass = readAuthBypass()
  const roles = useEffectiveRoles()
  const { data: reachable } = useQuery({ ...userQueries.myOrganizations(), enabled: !bypass })

  return useMemo(() => {
    if (bypass) return UNRESTRICTED
    if (!orgId || !roles || !reachable) return NO_PERMISSIONS
    return permissionsIn(roles, orgId, new Map(reachable.map((org) => [org.id, org.parentId])))
  }, [bypass, orgId, roles, reachable])
}
