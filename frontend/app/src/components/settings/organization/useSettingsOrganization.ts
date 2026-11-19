import { getRouteApi } from '@tanstack/react-router'
import type { OrganizationResponse } from '@/api/backendApi'
import { satisfies, type PermissionRequirement, type Permissions } from '@/lib/auth/permissions'
import { usePermissionsIn } from '@/lib/auth/usePermissionsIn'
import { useSettingsOrganizations } from './useSettingsOrganizations'

const organizationRoute = getRouteApi('/_protected/settings/organization/$orgId')

export interface SettingsOrganization {
  orgId: string
  /** See `useSettingsOrganizations`: member counts are real only for `readableIds`. */
  organizations: OrganizationResponse[]
  readableIds: ReadonlySet<string>
  perms: Permissions
  can: (required: PermissionRequirement) => boolean
}

export function useSettingsOrganization(): SettingsOrganization {
  const { orgId } = organizationRoute.useParams()
  const { organizations = [], readableIds } = useSettingsOrganizations()
  const perms = usePermissionsIn(orgId)
  return {
    orgId,
    organizations,
    readableIds,
    perms,
    can: (required) => satisfies(perms, required),
  }
}
