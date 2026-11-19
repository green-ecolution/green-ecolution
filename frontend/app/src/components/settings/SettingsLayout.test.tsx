import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import { render, screen, within } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import SettingsLayout from './SettingsLayout'

const state = vi.hoisted(
  (): {
    pathname: string
    perms: Permissions
    home: string | null
    navigate: ReturnType<typeof vi.fn>
  } => ({
    pathname: '/settings/organization/amt/members',
    perms: 'unrestricted',
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
vi.mock('@/lib/auth/usePermissionsIn', () => ({ usePermissionsIn: () => state.perms }))
vi.mock('@/lib/auth/scope/effectiveRoles', () => ({ useHomeOrganizationId: () => state.home }))
vi.mock('@/lib/auth/scope/adminMode', () => ({ useAdminMode: () => false }))
vi.mock('@tanstack/react-query', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@tanstack/react-query')>()),
  useQuery: ({ queryKey }: { queryKey: unknown[] }) => {
    if (queryKey[0] === 'organizations')
      return {
        data: [
          { id: 'amt', name: 'Grünflächenamt', memberCount: 3 },
          { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
        ],
      }
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
    state.home = 'amt'
    state.navigate.mockReset()
  })

  it('groups the entries under "Mein Konto" and "Einstellungen für"', () => {
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.getByText('Mein Konto')).toBeInTheDocument()
    expect(screen.getByText('Einstellungen für')).toBeInTheDocument()
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

  it('hides the organization group without any organization to manage', () => {
    state.pathname = '/settings/profile'
    state.home = null
    render(<SettingsLayout>content</SettingsLayout>)
    expect(screen.queryByText('Einstellungen für')).not.toBeInTheDocument()
  })
})
