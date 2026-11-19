import { useQuery } from '@tanstack/react-query'
import type { ReachableOrganizationResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '../runtimeConfig'
import { buildOrganizationRows } from './organizationRows'
import { useOrganizationScope } from './organizationScopeStore'

/**
 * The organization the settings open for: the scope, else the user's own, else
 * the topmost reachable one. Each candidate must still be reachable, so a
 * deleted scope organization falls through instead of being linked.
 */
export function resolveSettingsOrganizationId(
  scope: string | null,
  ownId: string | null | undefined,
  reachable: ReachableOrganizationResponse[],
): string | null {
  const isReachable = (id: string | null | undefined): id is string =>
    !!id && reachable.some((org) => org.id === id)
  if (isReachable(scope)) return scope
  if (isReachable(ownId)) return ownId
  return buildOrganizationRows(reachable)[0]?.id ?? null
}

/** `undefined` while loading; the auth bypass has no own organization. */
export function useSettingsHomeOrganizationId(): string | null | undefined {
  const bypass = readAuthBypass()
  const scope = useOrganizationScope()
  const { data: reachable } = useQuery(userQueries.myOrganizations())
  const { data: me } = useQuery({ ...userQueries.me(), enabled: !bypass })
  if (!reachable) return undefined
  if (bypass) return resolveSettingsOrganizationId(scope, undefined, reachable)
  if (!me) return undefined
  return resolveSettingsOrganizationId(scope, me.organization?.id, reachable)
}
