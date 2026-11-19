import { createFileRoute } from '@tanstack/react-router'
import OrganizationAreasPage from '@/components/settings/organization/OrganizationAreasPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/areas')(
  guardedRoute(['setting:read'], {
    component: OrganizationAreasPage,
    loader: () => ({ crumb: { titleKey: 'settingsAreas' as const } }),
  }),
)
