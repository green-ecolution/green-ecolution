import { createFileRoute } from '@tanstack/react-router'
import OrganizationOverviewPage from '@/components/settings/organization/OrganizationOverviewPage'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/')({
  component: OrganizationOverviewPage,
})
