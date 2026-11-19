import { createFileRoute } from '@tanstack/react-router'
import MembersPage from '@/components/settings/members/MembersPage'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/members')({
  component: MembersPage,
  loader: () => ({ crumb: { titleKey: 'settingsMembers' as const } }),
})
