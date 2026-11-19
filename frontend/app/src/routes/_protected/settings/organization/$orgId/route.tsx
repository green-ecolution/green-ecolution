import { createFileRoute } from '@tanstack/react-router'
import type { ReachableOrganizationResponse } from '@/api/backendApi'
import { userQueries } from '@/api/queries'
import SettingsOrganizationOutlet from '@/components/settings/organization/SettingsOrganizationOutlet'
import { EntityNotFoundError, entityNotFoundOnMiss } from '@/lib/router'

// The reachable list needs no permission; GET /organizations/{id} would demand
// organization:read, which pages such as stations or roles do not.
export const Route = createFileRoute('/_protected/settings/organization/$orgId')({
  loader: async ({ context: { queryClient }, params: { orgId } }) => {
    const find = (list: ReachableOrganizationResponse[]) => list.find((org) => org.id === orgId)
    // A cached list predates an organization created a moment ago, so a miss
    // asks the server once more before it counts.
    const organization =
      find(await queryClient.ensureQueryData(userQueries.myOrganizations())) ??
      find(await queryClient.fetchQuery({ ...userQueries.myOrganizations(), staleTime: 0 }))
    if (!organization) throw new EntityNotFoundError(`Organization ${orgId} is not reachable`)
    return { crumb: { title: organization.name } }
  },
  errorComponent: entityNotFoundOnMiss({
    entityName: { key: 'settings:organization.entityName' },
    backTo: '/settings/organization',
    backLabel: { key: 'settings:organization.notFoundBackLabel' },
  }),
  component: SettingsOrganizationOutlet,
})
