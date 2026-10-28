import { useQuery, type QueryClient } from '@tanstack/react-query'
import type { UserResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '../runtimeConfig'
import { getOrganizationScope, useOrganizationScope } from './organizationScopeStore'

/** The root is the only organization without a parent. */
export const isRootMember = (me: UserResponse | undefined): boolean =>
  me?.organization != null && me.organization.parentId == null

/**
 * Admin mode: a root member working without a scope sees the instance, not a
 * tenant. It needs a real login; the demo bypass has no home organization, so
 * the demo always shows the operational app.
 */
export function useAdminMode(): boolean {
  const bypass = readAuthBypass()
  const scope = useOrganizationScope()
  const { data: me } = useQuery({ ...userQueries.me(), enabled: !bypass })
  return !bypass && scope === null && isRootMember(me)
}

/** The route-guard twin of `useAdminMode`, for `beforeLoad`. */
export async function isAdminMode(queryClient: QueryClient): Promise<boolean> {
  if (readAuthBypass() || getOrganizationScope() !== null) return false
  const me = await queryClient.ensureQueryData(userQueries.me())
  return isRootMember(me)
}
