import { createFileRoute } from '@tanstack/react-router'
import OrganizationAreasPage from '@/components/settings/organization/OrganizationAreasPage'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/areas')({
  component: OrganizationAreasPage,
  loader: () => ({ crumb: { titleKey: 'settingsAreas' as const } }),
})
