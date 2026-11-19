import { createFileRoute } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import SettingsOrganizationOutlet from '@/components/settings/organization/SettingsOrganizationOutlet'
import { entityNotFound } from '@/lib/router'

// The reachable list needs no permission; GET /organizations/{id} would demand
// organization:read, which pages such as stations or roles do not.
export const Route = createFileRoute('/_protected/settings/organization/$orgId')({
  loader: async ({ context: { queryClient }, params: { orgId } }) => {
    const reachable = await queryClient.ensureQueryData(userQueries.myOrganizations())
    const organization = reachable.find((org) => org.id === orgId)
    if (!organization) throw new Error(`Organization ${orgId} is not reachable`)
    return { crumb: { title: organization.name } }
  },
  errorComponent: entityNotFound({
    entityName: { key: 'settings:organization.entityName' },
    backTo: '/settings/organization',
    backLabel: { key: 'settings:organization.notFoundBackLabel' },
  }),
  component: SettingsOrganizationOutlet,
})
