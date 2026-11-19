import { createFileRoute, redirect } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import { buildOrganizationRows } from '@/lib/auth/scope/organizationRows'
import { getOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { ForbiddenError, forbiddenErrorComponent, requirePermission } from '@/lib/router'

// The entry without an id: the scope organization, else the user's own, else
// the topmost reachable one (the auth bypass has no own organization).
export const Route = createFileRoute('/_protected/settings/organization/')({
  beforeLoad: async ({ context }) => {
    await requirePermission(['organization:read'])({ context })
    const { queryClient } = context
    const reachable = await queryClient.ensureQueryData(userQueries.myOrganizations())
    const ownId = readAuthBypass()
      ? undefined
      : (await queryClient.ensureQueryData(userQueries.me())).organization?.id
    const own = reachable.some((org) => org.id === ownId) ? ownId : undefined
    const orgId = getOrganizationScope() ?? own ?? buildOrganizationRows(reachable)[0]?.id
    if (!orgId) throw new ForbiddenError()
    throw redirect({ to: '/settings/organization/$orgId', params: { orgId } })
  },
  errorComponent: forbiddenErrorComponent(),
})
