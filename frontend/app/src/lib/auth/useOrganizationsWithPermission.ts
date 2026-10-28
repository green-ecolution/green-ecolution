import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import { organizationQueries } from '@/api/queries'
import { byOrgName, orgsWithPermission } from './organizationScope'
import type { Permission } from './permissions'
import { readAuthBypass } from './runtimeConfig'
import { useEffectiveRoles } from './scope/effectiveRoles'
import { useHasPermission } from './useHasPermission'

/**
 * Organizations in which the user holds `permission`, sorted by name.
 *
 * The candidate list needs the organization tree, which requires
 * `organization:read`. Without it the list stays empty.
 */
export function useOrganizationsWithPermission(permission: Permission): OrganizationResponse[] {
  const bypass = readAuthBypass()
  const canReadOrganizations = useHasPermission(['organization:read'])
  const roles = useEffectiveRoles()
  const { data: orgs } = useQuery({
    ...organizationQueries.list(),
    enabled: canReadOrganizations,
  })

  return useMemo(() => {
    if (!orgs) return []
    // Bypass mirrors the backend's unrestricted mode: every visible org counts.
    if (bypass) return [...orgs].sort(byOrgName)
    return orgsWithPermission(orgs, roles ?? [], permission)
  }, [orgs, roles, bypass, permission])
}
