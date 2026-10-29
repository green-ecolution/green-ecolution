import { createFileRoute } from '@tanstack/react-router'
import { organizationQueries } from '@/api/queries'
import TenantOverview from '@/components/admin/TenantOverview'
import { pendingLoading, prefetch } from '@/lib/router'

export const Route = createFileRoute('/_protected/admin/')({
  component: TenantOverview,
  pendingComponent: pendingLoading({ key: 'admin:tenants.loadingLabel' }),
  loader: ({ context: { queryClient } }) => {
    prefetch(queryClient, organizationQueries.list(), 'organizationQueries.list')
    prefetch(
      queryClient,
      organizationQueries.resourceCounts(),
      'organizationQueries.resourceCounts',
    )
  },
})
