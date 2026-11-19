import { createFileRoute } from '@tanstack/react-router'
import MembersPage from '@/components/settings/members/MembersPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/members')(
  guardedRoute(['user:read'], {
    component: MembersPage,
    loader: () => ({ crumb: { titleKey: 'settingsMembers' as const } }),
  }),
)
