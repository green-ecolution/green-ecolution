import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import type {
  OrganizationResponse,
  ReachableOrganizationResponse,
  RoleResponse,
} from '@green-ecolution/backend-client'
import { render, screen, userEvent, within } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import SettingsLayout from './SettingsLayout'

const VISIBLE: OrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3 },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
]
const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt' },
]

const roleWith = (permissions: string[]): RoleResponse =>
  ({ id: 'r', name: 'Rolle', organizationId: 'amt', permissions }) as RoleResponse

const state = vi.hoisted(
  (): {
    pathname: string
    perms: Permissions
    permsByOrg: Partial<Record<string, Permissions>>
    adminMode: boolean
    roles: RoleResponse[]
    home: string | null | undefined
    navigate: ReturnType<typeof vi.fn>
  } => ({
    pathname: '/settings/organization/amt/members',
    perms: 'unrestricted',
    permsByOrg: {},
    adminMode: false,
    roles: [],
    home: 'amt',
    navigate: vi.fn(),
  }),
)

vi.mock('@tanstack/react-router', () => ({
  Link: ({
    to,
    params,
    children,
  }: {
    to: string
    params?: Record<string, string>
    children: ReactNode
  }) => (
    <a
      href={Object.entries(params ?? {}).reduce(
        (path, [key, value]) => path.replace(`$${key}`, value),
        to,
      )}
    >
      {children}
    </a>
  ),
  useLocation: () => ({ pathname: state.pathname }),
  useNavigate: () => state.navigate,
}))
vi.mock('@/lib/auth/usePermissionsIn', () => ({
  usePermissionsResolver: () => (orgId: string | null) =>
    (orgId === null ? undefined : state.permsByOrg[orgId]) ?? state.perms,
}))
vi.mock('@/lib/auth/scope/settingsOrganization', () => ({
  useSettingsHomeOrganizationId: () => state.home,
}))
vi.mock('@/lib/auth/scope/effectiveRoles', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/lib/auth/scope/effectiveRoles')>()),
  useEffectiveRoles: () => state.roles,
}))
vi.mock('@/lib/auth/scope/adminMode', () => ({ useAdminMode: () => state.adminMode }))
// Like the backend, GET /organizations answers nothing without organization:read.
vi.mock('@tanstack/react-query', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@tanstack/react-query')>()),
  useQuery: ({ queryKey, enabled }: { queryKey: unknown[]; enabled?: boolean }) => {
    if (enabled === false) return { data: undefined }
    const key = queryKey.join('/')
    if (key === 'organizations') {
      const canRead = state.roles.some((role) => role.permissions.includes('organization:read'))
      return { data: canRead ? VISIBLE : undefined }
    }
    if (key === 'users/me/organizations') return { data: REACHABLE }
    return {
      data: {
        items: [
          { name: 'plugins', enabled: true },
          { name: 'routing', enabled: true },
        ],
      },
    }
  },
}))

describe('SettingsLayout', () => {
  beforeEach(() => {
    state.pathname = '/settings/organization/amt/members'
    state.perms = 'unrestricted'
    state.permsByOrg = {}
    state.adminMode = false
    state.roles = [roleWith(['organization:read'])]
    state.home = 'amt'
    state.navigate.mockReset()
  })

  it('groups the entries under "Mein Konto" and "Einstellungen für"', () => {
    render(<SettingsLayout>content</SettingsLayout>)
    expect(
      within(screen.getByRole('list', { name: 'Mein Konto' })).getByRole('link', {
        name: 'Profil',
      }),
    ).toBeInTheDocument()
    expect(
      within(screen.getByRole('list', { name: 'Einstellungen für' })).getByRole('link', {
        name: 'Überblick',
      }),
    ).toBeInTheDocument()
  })

  it('reads the counts as words and hides the bare numbers', () => {
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByRole('link', { name: 'Struktur 1 Unterorganisation' })).toBeInTheDocument()
    const members = screen.getByRole('link', { name: 'Mitarbeitende 7 Personen' })
    expect(within(members).getByText('7')).toHaveAttribute('aria-hidden', 'true')
  })

  it('links the organization entries to the organization in the path', () => {
    state.pathname = '/settings/organization/nord/roles'
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByRole('link', { name: /Mitarbeitende/ })).toHaveAttribute(
      'href',
      '/settings/organization/nord/members',
    )
  })

  it('falls back to the home organization on the account pages', () => {
    state.pathname = '/settings/profile'
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByRole('link', { name: 'Überblick' })).toHaveAttribute(
      'href',
      '/settings/organization/amt',
    )
  })

  it('falls back to the resolved organization without an own one', () => {
    state.pathname = '/settings/profile'
    state.home = 'nord'
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByRole('link', { name: 'Überblick' })).toHaveAttribute(
      'href',
      '/settings/organization/nord',
    )
  })

  it('hides entries the user has no right to in this organization', () => {
    state.perms = new Set(['organization:read'])
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.queryByRole('link', { name: /Rollen/ })).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Struktur/ })).toBeInTheDocument()
  })

  it('shows the number of sub-organizations and members', () => {
    render(<SettingsLayout>content</SettingsLayout>)
    expect(
      within(screen.getByRole('link', { name: /Struktur/ })).getByText('1'),
    ).toBeInTheDocument()
    expect(
      within(screen.getByRole('link', { name: /Mitarbeitende/ })).getByText('7'),
    ).toBeInTheDocument()
  })

  it('works for a role without organization:read', () => {
    state.pathname = '/settings/organization/amt/stations'
    state.roles = [roleWith(['start_point:read'])]
    state.perms = new Set(['start_point:read'])
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByText('Einstellungen für')).toBeInTheDocument()
    // One picker for narrow screens, one inside the group from lg up.
    expect(
      screen.getAllByRole('button', { name: 'Organisation wechseln, aktuell Grünflächenamt' }),
    ).toHaveLength(2)
    expect(screen.getByRole('link', { name: 'Stationen' })).toHaveAttribute(
      'href',
      '/settings/organization/amt/stations',
    )
  })

  it('counts the sub-organizations but not the members without organization:read', () => {
    state.roles = [roleWith(['user:read'])]
    state.perms = new Set(['user:read', 'organization:read'])
    render(<SettingsLayout>content</SettingsLayout>)
    expect(
      within(screen.getByRole('link', { name: /Struktur/ })).getByText('1'),
    ).toBeInTheDocument()
    expect(within(screen.getByRole('link', { name: /Mitarbeitende/ })).queryByText(/\d/)).toBeNull()
  })

  it('hides the organization group without any organization to manage', () => {
    state.pathname = '/settings/profile'
    state.home = null
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.queryByText('Einstellungen für')).not.toBeInTheDocument()
  })

  it('hides the organization group while the home organization loads', () => {
    state.pathname = '/settings/profile'
    state.home = undefined
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.queryByText('Einstellungen für')).not.toBeInTheDocument()
  })

  it('hides stations at the instance root in admin mode', () => {
    state.adminMode = true
    state.pathname = '/settings/organization/amt'
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.queryByRole('link', { name: 'Stationen' })).not.toBeInTheDocument()
  })

  it('shows stations for a tenant picked in admin mode', () => {
    state.adminMode = true
    state.pathname = '/settings/organization/nord'
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByRole('link', { name: 'Stationen' })).toBeInTheDocument()
  })
})

describe('SettingsLayout organization switch', () => {
  beforeEach(() => {
    state.pathname = '/settings/organization/amt/roles'
    state.perms = 'unrestricted'
    state.permsByOrg = {}
    state.adminMode = false
    state.roles = [roleWith(['organization:read'])]
    state.home = 'amt'
    state.navigate.mockReset()
  })

  const switchTo = async (name: RegExp) => {
    await userEvent.click(screen.getAllByRole('button', { name: /Organisation wechseln/ })[0])
    await userEvent.click(screen.getByRole('option', { name }))
  }

  it('stays on the same section where the target allows it', async () => {
    state.permsByOrg = { nord: new Set(['role:read']) }
    render(<SettingsLayout>content</SettingsLayout>)
    await switchTo(/Stadtgärtnerei Nord/)
    expect(state.navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId/roles',
      params: { orgId: 'nord' },
    })
  })

  it('moves to the first visible section where the target forbids the current one', async () => {
    state.permsByOrg = { nord: new Set(['user:read', 'start_point:read']) }
    render(<SettingsLayout>content</SettingsLayout>)
    await switchTo(/Stadtgärtnerei Nord/)
    expect(state.navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId/members',
      params: { orgId: 'nord' },
    })
  })

  it('maps a plugin detail to the plugin list', async () => {
    state.pathname = '/settings/organization/amt/plugins/kataster'
    render(<SettingsLayout>content</SettingsLayout>)
    await switchTo(/Stadtgärtnerei Nord/)
    expect(state.navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId/plugins',
      params: { orgId: 'nord' },
    })
  })
})
