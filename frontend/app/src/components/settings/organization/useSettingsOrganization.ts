import { getRouteApi } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import type { OrganizationResponse } from '@/api/backendApi'
import { organizationQueries } from '@/api/queries'
import { satisfies, type PermissionRequirement, type Permissions } from '@/lib/auth/permissions'
import { usePermissionsIn } from '@/lib/auth/usePermissionsIn'

const organizationRoute = getRouteApi('/_protected/settings/organization/$orgId')

export interface SettingsOrganization {
  orgId: string
  /** Every organization the user can see, as GET /organizations returns them. */
  organizations: OrganizationResponse[]
  perms: Permissions
  can: (required: PermissionRequirement) => boolean
}

export function useSettingsOrganization(): SettingsOrganization {
  const { orgId } = organizationRoute.useParams()
  const { data: organizations = [] } = useQuery(organizationQueries.list())
  const perms = usePermissionsIn(orgId)
  return { orgId, organizations, perms, can: (required) => satisfies(perms, required) }
}
