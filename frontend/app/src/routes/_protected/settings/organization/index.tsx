import { createFileRoute, redirect } from '@tanstack/react-router'
import { infoQueries, userQueries } from '@/api/queries'
import { landingSection } from '@/components/settings/settingsNav'
import { SECTION_ROUTES } from '@/components/settings/settingsSections'
import { permissionsIn } from '@/lib/auth/organizationScope'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { isAdminMode } from '@/lib/auth/scope/adminMode'
import { ensureEffectiveRoles } from '@/lib/auth/scope/effectiveRoles'
import { getOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { resolveSettingsOrganizationId } from '@/lib/auth/scope/settingsOrganization'
import { ForbiddenError, forbiddenErrorComponent } from '@/lib/router'

// No permission of its own: it opens the first section the user may see in
// the resolved organization, the same choice the organization switch makes.
export const Route = createFileRoute('/_protected/settings/organization/')({
  beforeLoad: async ({ context: { queryClient } }) => {
    const bypass = readAuthBypass()
    // fetchQuery, not ensureQueryData: right after a delete the cached list is
    // invalidated but may still hold the deleted organization.
    const reachable = await queryClient.fetchQuery(userQueries.myOrganizations())
    const ownId = bypass
      ? undefined
      : (await queryClient.ensureQueryData(userQueries.me())).organization?.id
    const orgId = resolveSettingsOrganizationId(getOrganizationScope(), ownId, reachable)
    if (!orgId) throw new ForbiddenError()

    const perms = bypass
      ? UNRESTRICTED
      : permissionsIn(
          await ensureEffectiveRoles(queryClient),
          orgId,
          new Map(reachable.map((org) => [org.id, org.parentId])),
        )
    const services = await queryClient.ensureQueryData(infoQueries.services())
    const enabledFeatures = new Set(
      services.items.filter((item) => item.enabled).map((item) => item.name),
    )
    const atInstanceRoot =
      (await isAdminMode(queryClient)) &&
      reachable.some((org) => org.id === orgId && org.parentId == null)

    const section = landingSection(perms, enabledFeatures, atInstanceRoot)
    if (!section) throw new ForbiddenError()
    throw redirect({ to: SECTION_ROUTES[section], params: { orgId } })
  },
  errorComponent: forbiddenErrorComponent(),
})
