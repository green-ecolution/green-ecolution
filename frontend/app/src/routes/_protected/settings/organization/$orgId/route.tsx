import { createFileRoute } from '@tanstack/react-router'
import { organizationQueries } from '@/api/queries'
import { entityRoute } from '@/lib/router'

// An organization the caller may not read answers 403; entityRoute shows that
// as "not found", the same way the backend hides scoped resources.
export const Route = createFileRoute('/_protected/settings/organization/$orgId')(
  entityRoute({
    key: 'organization',
    query: organizationQueries.byId,
    idParam: 'orgId',
    title: (organization) => organization.name,
    notFound: {
      entityName: { key: 'settings:organization.entityName' },
      backTo: '/settings/organization',
      backLabel: { key: 'settings:organization.notFoundBackLabel' },
    },
  }),
)
