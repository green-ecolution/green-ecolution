import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor, within } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type {
  OrganizationDetailResponse,
  OrganizationResponse,
  UserResponse,
} from '@/api/backendApi'
import type { UpdateOrganizationVariables } from '@/hooks/useOrganizationMutations'

// The instance root ('root') sits above the user's own organization ('amt') and
// must stay out of the tree. parentId is left off entirely rather than set to
// null, because the generated client turns JSON null into undefined.
const orgs: OrganizationResponse[] = [
  { id: 'root', name: 'Stadt Flensburg', memberCount: 1 },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
  { id: 'juergensby', name: 'Team Jürgensby', memberCount: 5, parentId: 'nord' },
]

const details: Record<string, OrganizationDetailResponse> = {
  root: { id: 'root', name: 'Stadt Flensburg', memberCount: 1 },
  amt: { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  nord: { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  duburg: { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
  juergensby: { id: 'juergensby', name: 'Team Jürgensby', memberCount: 5, parentId: 'nord' },
}

const member = (id: string, firstName: string, lastName: string, orgId: string): UserResponse => ({
  id,
  firstName,
  lastName,
  username: id,
  email: `${id}@flensburg.de`,
  emailVerified: true,
  employeeId: id,
  phoneNumber: '',
  avatarUrl: '',
  createdAt: '2026-01-02T00:00:00Z',
  drivingLicenses: [],
  roles: [],
  status: 'available',
  organization: orgs.find((org) => org.id === orgId),
})

const users: UserResponse[] = [
  member('u-anna', 'Anna', 'Ahlmann', 'amt'),
  member('u-bo', 'Bo', 'Boysen', 'nord'),
]

const permissions = vi.fn((): Permissions => UNRESTRICTED)
vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => permissions() }))

// jsdom's matchMedia mock always reports non-desktop, which would wrap the detail
// panel in a Drawer and leave both panes unqueryable. Default to the desktop
// two-pane layout.
const isDesktop = vi.fn((): boolean => true)
vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => isDesktop() }))

const createMutate = vi.fn()
const updateMutate = vi.fn()
const deleteMutate = vi.fn()
vi.mock('@/hooks/useOrganizationMutations', () => ({
  useOrganizationMutations: () => ({
    createOrganization: { mutate: createMutate, isPending: false, error: null, reset: vi.fn() },
    updateOrganization: { mutate: updateMutate, isPending: false, error: null, reset: vi.fn() },
    deleteOrganization: { mutate: deleteMutate, isPending: false, error: null, reset: vi.fn() },
  }),
}))

vi.mock('@tanstack/react-router', () => ({
  useBlocker: () => ({ status: 'idle' }),
  Link: ({ children }: { children: ReactNode }) => <a href="#link">{children}</a>,
}))

const ownOrgId = vi.fn((): string => 'amt')

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[] }) => {
      const [scope, second] = options.queryKey
      if (scope === 'users' && second === 'me') {
        return {
          data: {
            id: 'u-anna',
            roles: [],
            organization: orgs.find((org) => org.id === ownOrgId()),
          },
          isLoading: false,
        }
      }
      if (scope === 'users') return { data: { data: users }, isLoading: false }
      if (scope === 'organizations' && second === undefined) {
        return { data: orgs, isLoading: false }
      }
      if (scope === 'organizations') {
        return { data: details[second as string], isLoading: false }
      }
      return { data: undefined, isLoading: false }
    },
  }
})

const { default: OrganizationPage } = await import('./OrganizationPage')

/** Scopes queries to the tree pane so detail-pane duplicates don't interfere. */
const tree = () => {
  const region = screen
    .getByRole('heading', { name: 'Organisationsstruktur' })
    .closest('div')?.parentElement
  if (!region) throw new Error('tree pane not found')
  return within(region)
}

const treeRow = (name: RegExp) => {
  const row = tree().getByRole('button', { name }).closest('li')
  if (!row) throw new Error(`tree row ${String(name)} not found`)
  return within(row)
}

describe('OrganizationPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(UNRESTRICTED)
    isDesktop.mockReturnValue(true)
    ownOrgId.mockReturnValue('amt')
  })

  it('renders the tree rooted at the own organization', () => {
    render(<OrganizationPage />)

    expect(tree().getByRole('button', { name: /Grünflächenamt/ })).toBeInTheDocument()
    expect(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ })).toBeInTheDocument()
    // The instance root above the own organization must not be reachable at all.
    expect(screen.queryByText('Stadt Flensburg')).not.toBeInTheDocument()
  })

  it('shows the member count of the whole subtree per node', () => {
    render(<OrganizationPage />)

    // 4 own + 6 Duburg + 5 Jürgensby
    expect(tree().getByText('15 Personen · 2 Teams')).toBeInTheDocument()
    expect(tree().queryByText(/^4 Personen/)).not.toBeInTheDocument()
    // 3 own + the 15 below Stadtgärtnerei Nord
    expect(tree().getByText('18 Personen · 1 Teams')).toBeInTheDocument()
  })

  it('expands and collapses a node without selecting it', async () => {
    render(<OrganizationPage />)

    expect(tree().queryByRole('button', { name: /Team Duburg/ })).not.toBeInTheDocument()

    await userEvent.click(
      treeRow(/Stadtgärtnerei Nord/).getByRole('button', { name: 'Aufklappen' }),
    )

    expect(tree().getByRole('button', { name: /Team Duburg/ })).toBeInTheDocument()
    expect(tree().getByRole('button', { name: /Grünflächenamt/ })).toHaveAttribute(
      'aria-current',
      'true',
    )
    expect(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ })).toHaveAttribute(
      'aria-current',
      'false',
    )

    await userEvent.click(treeRow(/Stadtgärtnerei Nord/).getByRole('button', { name: 'Zuklappen' }))

    expect(tree().queryByRole('button', { name: /Team Duburg/ })).not.toBeInTheDocument()
  })

  it('blocks saving while the address is partial', async () => {
    render(<OrganizationPage />)
    await userEvent.click(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ }))

    await userEvent.type(screen.getByRole('textbox', { name: 'PLZ' }), '24939')

    expect(screen.getByRole('button', { name: 'Speichern' })).toBeDisabled()
    expect(screen.getByText('Straße fehlt')).toBeInTheDocument()
    expect(screen.getByText('Ort fehlt')).toBeInTheDocument()
    expect(updateMutate).not.toHaveBeenCalled()
  })

  it('saves name, address and contact person together', async () => {
    render(<OrganizationPage />)
    await userEvent.click(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ }))

    await userEvent.type(
      screen.getByRole('textbox', { name: 'Straße und Hausnummer' }),
      'Nordergraben 12',
    )
    await userEvent.type(screen.getByRole('textbox', { name: 'PLZ' }), '24937')
    await userEvent.type(screen.getByRole('textbox', { name: 'Ort' }), 'Flensburg')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    expect(updateMutate.mock.calls[0][0] as UpdateOrganizationVariables).toEqual({
      orgId: 'nord',
      name: 'Stadtgärtnerei Nord',
      address: { street: 'Nordergraben 12', postalCode: '24937', city: 'Flensburg' },
      contactPersonId: null,
    })
  })

  it('renders the own organization read-only when it is the instance root', () => {
    ownOrgId.mockReturnValue('root')
    render(<OrganizationPage />)

    expect(
      screen.getByText('Die oberste Organisation dieser Instanz kann nicht bearbeitet werden.'),
    ).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Name' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Speichern' })).not.toBeInTheDocument()
  })

  it('hides the create button without organization:create', () => {
    permissions.mockReturnValue(new Set(['organization:read', 'organization:update', 'user:read']))
    render(<OrganizationPage />)

    // The card that hosts the button is there; only the action is gone.
    expect(screen.getByText('Untergeordnete Organisationen')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Neu' })).not.toBeInTheDocument()
    expect(
      screen.queryByRole('button', { name: /Unterorganisation anlegen/ }),
    ).not.toBeInTheDocument()
  })

  it('hides the delete action without organization:delete', async () => {
    permissions.mockReturnValue(
      new Set(['organization:read', 'organization:update', 'organization:create', 'user:read']),
    )
    render(<OrganizationPage />)
    await userEvent.click(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ }))

    // A sub-organization would be deletable; the editable pane proves it rendered.
    expect(screen.getByRole('textbox', { name: 'Name' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Organisation löschen/ })).not.toBeInTheDocument()
    expect(screen.queryByText(/Eine Organisation lässt sich nur löschen/)).not.toBeInTheDocument()
  })

  it('hides the members card without user:read', () => {
    permissions.mockReturnValue(
      new Set([
        'organization:read',
        'organization:update',
        'organization:create',
        'organization:delete',
      ]),
    )
    render(<OrganizationPage />)

    expect(screen.getByText('Kontaktperson')).toBeInTheDocument()
    expect(screen.queryByText('Zugewiesene Mitarbeitende')).not.toBeInTheDocument()
  })

  it('asks before discarding unsaved changes when selecting another node', async () => {
    render(<OrganizationPage />)
    await userEvent.click(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ }))
    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), ' Ost')

    await userEvent.click(tree().getByRole('button', { name: /Grünflächenamt/ }))

    expect(await screen.findByText('Änderungen verwerfen?')).toBeInTheDocument()
    // The modal hides the pane from the a11y tree, so probe the still-mounted draft.
    expect(screen.getByDisplayValue('Stadtgärtnerei Nord Ost')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Verwerfen' }))

    await waitFor(() =>
      expect(screen.queryByDisplayValue('Stadtgärtnerei Nord Ost')).not.toBeInTheDocument(),
    )
    expect(screen.getByRole('textbox', { name: 'Name' })).toHaveValue('Grünflächenamt')
  })
})
