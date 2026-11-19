import { createFileRoute, redirect } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import { getOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { resolveSettingsOrganizationId } from '@/lib/auth/scope/settingsOrganization'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { ForbiddenError, forbiddenErrorComponent, requirePermission } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/')({
  beforeLoad: async ({ context }) => {
    await requirePermission(['organization:read'])({ context })
    const { queryClient } = context
    const reachable = await queryClient.ensureQueryData(userQueries.myOrganizations())
    const ownId = readAuthBypass()
      ? undefined
      : (await queryClient.ensureQueryData(userQueries.me())).organization?.id
    const orgId = resolveSettingsOrganizationId(getOrganizationScope(), ownId, reachable)
    if (!orgId) throw new ForbiddenError()
    throw redirect({ to: '/settings/organization/$orgId', params: { orgId } })
  },
  errorComponent: forbiddenErrorComponent(),
})
