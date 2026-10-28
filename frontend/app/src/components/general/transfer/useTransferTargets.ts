import { useMemo } from 'react'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import type { Permission } from '@/lib/auth/permissions'
import { useOrganizationsWithPermission } from '@/lib/auth/useOrganizationsWithPermission'

/**
 * Organizations an object owned by `currentOrganizationId` may be moved to.
 *
 * The backend requires the update right in both the source and the target
 * organization, so only organizations where the user holds `permission` are
 * offered. The current owner is left out: moving there is a no-op.
 */
export function useTransferTargets(
  permission: Permission,
  currentOrganizationId: string,
): OrganizationResponse[] {
  const candidates = useOrganizationsWithPermission(permission)
  return useMemo(
    () => candidates.filter((org) => org.id !== currentOrganizationId),
    [candidates, currentOrganizationId],
  )
}
