import { createFileRoute } from '@tanstack/react-router'
import OrganizationStructurePage from '@/components/settings/organization/OrganizationStructurePage'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/structure')({
  component: OrganizationStructurePage,
  loader: () => ({ crumb: { titleKey: 'settingsStructure' as const } }),
})
