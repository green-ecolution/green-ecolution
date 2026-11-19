import { createFileRoute } from '@tanstack/react-router'
import RolesPage from '@/components/settings/roles/RolesPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/roles')(
  guardedRoute(['role:read'], {
    component: RolesPage,
    loader: () => ({ crumb: { titleKey: 'settingsRoles' as const } }),
  }),
)
