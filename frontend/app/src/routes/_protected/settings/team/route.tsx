import { createFileRoute, Link, Outlet } from '@tanstack/react-router'
import { guardedRoute } from '@/lib/router'
import { useHasPermission } from '@/lib/auth/useHasPermission'

const TABS = [
  { to: '/settings/team/members', label: 'Mitarbeitende', permission: ['user:read'] as const },
  { to: '/settings/team/roles', label: 'Rollen', permission: ['role:read'] as const },
]

const TeamTabs = () => {
  const canReadUsers = useHasPermission(['user:read'])
  const canReadRoles = useHasPermission(['role:read'])
  const allowed = { '/settings/team/members': canReadUsers, '/settings/team/roles': canReadRoles }

  return (
    <>
      <nav
        aria-label="Team und Rollen"
        className="sticky top-[var(--settings-header-top)] z-10 mb-6 flex gap-1 border-b border-dark-200 bg-dark-50"
      >
        {TABS.filter((tab) => allowed[tab.to as keyof typeof allowed]).map((tab) => (
          <Link
            key={tab.to}
            to={tab.to}
            className="-mb-px border-b-2 border-transparent px-4 py-2 font-nunito-sans text-sm text-dark-600 transition-colors hover:text-dark data-[status=active]:border-green-dark data-[status=active]:font-semibold data-[status=active]:text-green-dark"
          >
            {tab.label}
          </Link>
        ))}
      </nav>
      <Outlet />
    </>
  )
}

export const Route = createFileRoute('/_protected/settings/team')(
  guardedRoute(['user:read', 'role:read'], {
    component: TeamTabs,
    loader: () => ({ crumb: { title: 'Team & Rollen' } }),
  }),
)
