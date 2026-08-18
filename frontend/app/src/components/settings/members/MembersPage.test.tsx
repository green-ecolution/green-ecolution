import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor, within } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'

const ORGS: OrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3 },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
]

const role = (id: string, name: string): RoleResponse => ({
  id,
  name,
  permissions: [],
  isTemplate: false,
})

const ROLES: RoleResponse[] = [role('r-lead', 'Einsatzleitung'), role('r-garden', 'Gärtner')]

// Keyed by org so the picker's org-scoping (a role only grants rights over its
// own organization's subtree) can be told apart from "returns some roles".
const ROLES_BY_ORG: Record<string, RoleResponse[]> = {
  amt: ROLES,
  nord: [role('r-driver', 'Fahrerin')],
}

// The list filter's endpoint spans the caller's whole visible subtree, so it
// includes "nord"'s role even though the signed-in caller belongs to "amt" —
// unlike ROLES_BY_ORG.amt, which only the assignment picker ever reads.
const VISIBLE_ROLES: RoleResponse[] = [...ROLES_BY_ORG.amt, ...ROLES_BY_ORG.nord]

const avatarOf = (id: string) => `https://cdn.example/${id}.png`

const member = (
  id: string,
  firstName: string,
  lastName: string,
  orgId: string | null,
  roles: RoleResponse[],
): UserResponse => ({
  id,
  firstName,
  lastName,
  username: id,
  email: `${id}@flensburg.de`,
  emailVerified: true,
  employeeId: 'EMP-1',
  phoneNumber: '+49 461 1',
  // A distinctive value, so a save that drops the avatar cannot pass unnoticed.
  avatarUrl: avatarOf(id),
  createdAt: '2026-01-02T00:00:00Z',
  drivingLicenses: [],
  roles,
  status: 'available',
  organization: ORGS.find((org) => org.id === orgId),
})

const users: UserResponse[] = [
  member('u-anna', 'Anna', 'Ahlmann', 'amt', [ROLES[0]]),
  member('u-bo', 'Bo', 'Boysen', 'nord', []),
  member('u-legacy', 'Cem', 'Demir', null, []),
]

const permissions = vi.fn((): Permissions => UNRESTRICTED)
vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => permissions() }))

// jsdom's matchMedia mock always reports non-desktop, which would wrap the
// detail panel in a Drawer and leave both panes unqueryable.
const isDesktop = vi.fn((): boolean => true)
vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => isDesktop() }))

const assignMutate = vi.fn()
const revokeMutate = vi.fn()
const setOrgMutate = vi.fn()
const updateMutate = vi.fn()
const assignError = vi.fn((): unknown => null)
const setOrgError = vi.fn((): unknown => null)
vi.mock('@/hooks/useUserMutations', () => ({
  useUserMutations: () => ({
    assignRole: { mutate: assignMutate, isPending: false, error: assignError(), reset: vi.fn() },
    revokeRole: { mutate: revokeMutate, isPending: false, error: null, reset: vi.fn() },
    setOrganization: {
      mutate: setOrgMutate,
      isPending: false,
      error: setOrgError(),
      reset: vi.fn(),
    },
    updateProfile: { mutate: updateMutate, isPending: false, error: null, reset: vi.fn() },
  }),
}))

const blockerStatus = vi.fn((): string => 'idle')
vi.mock('@tanstack/react-router', () => ({
  useBlocker: () => ({ status: blockerStatus(), proceed: vi.fn(), reset: vi.fn() }),
  Link: ({ children }: { children: ReactNode }) => <a href="#link">{children}</a>,
}))

const meId = vi.fn((): string => 'u-anna')

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[] }) => {
      const [scope, second, third] = options.queryKey
      if (scope === 'users' && second === 'me') {
        return { data: { id: meId(), roles: [], organization: ORGS[0] }, isLoading: false }
      }
      if (scope === 'users') {
        const params = (second ?? {}) as {
          query?: string
          organizationId?: string
          roleId?: string
        }
        const matches = users.filter((user) => {
          if (params.organizationId && user.organization?.id !== params.organizationId) return false
          if (params.roleId && !user.roles.some((r) => r.id === params.roleId)) return false
          if (params.query) {
            const needle = params.query.toLowerCase()
            return `${user.firstName} ${user.lastName} ${user.username}`
              .toLowerCase()
              .includes(needle)
          }
          return true
        })
        return {
          data: {
            data: matches,
            pagination: {
              currentPage: 1,
              perPage: 50,
              totalRecords: matches.length,
              totalPages: 1,
            },
          },
          isLoading: false,
        }
      }
      if (scope === 'organizations') return { data: ORGS, isLoading: false }
      if (scope === 'roles' && second === 'org') {
        const orgId = third as string
        return { data: ROLES_BY_ORG[orgId] ?? [], isLoading: false }
      }
      if (scope === 'roles' && second === 'visible') {
        return { data: VISIBLE_ROLES, isLoading: false }
      }
      return { data: undefined, isLoading: false }
    },
  }
})

const { default: MembersPage } = await import('./MembersPage')

const list = () => within(screen.getByRole('list', { name: 'Mitarbeitende' }))

const select = async (name: RegExp) => {
  await userEvent.click(list().getByRole('button', { name }))
}

const editEmployeeId = async (value: string) => {
  await userEvent.clear(screen.getByLabelText('Personalnummer'))
  await userEvent.type(screen.getByLabelText('Personalnummer'), value)
}

const openCombobox = async (name: string) => {
  await userEvent.click(screen.getByRole('combobox', { name }))
}

describe('MembersPage', () => {
  beforeEach(() => {
    // clearAllMocks keeps return values, so every error mock is reset explicitly.
    vi.clearAllMocks()
    // Nobody in the list is the signed-in user by default; the self-lock test
    // opts in explicitly.
    meId.mockReturnValue('u-admin')
    isDesktop.mockReturnValue(true)
    permissions.mockReturnValue(UNRESTRICTED)
    assignError.mockReturnValue(null)
    setOrgError.mockReturnValue(null)
  })

  it('lists every member', () => {
    render(<MembersPage />)

    expect(list().getByRole('button', { name: /Anna Ahlmann/ })).toBeInTheDocument()
    expect(list().getByRole('button', { name: /Bo Boysen/ })).toBeInTheDocument()
  })

  it('assigns a role immediately, without a save step', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await openCombobox('Rolle zuweisen')
    await userEvent.click(screen.getByRole('option', { name: 'Gärtner' }))

    expect(assignMutate).toHaveBeenCalledWith(
      expect.objectContaining({ userId: 'u-anna', roleId: 'r-garden' }),
      expect.anything(),
    )
  })

  it('offers only roles that are not assigned yet', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await openCombobox('Rolle zuweisen')

    expect(screen.queryByRole('option', { name: 'Einsatzleitung' })).not.toBeInTheDocument()
  })

  it("offers only the selected person's own organization roles, not another organization's", async () => {
    render(<MembersPage />)
    // Bo belongs to "nord", not the default org used everywhere else, so the
    // picker's role set can only come from the selected person's org, not a
    // hardcoded or own-org fallback.
    await select(/Bo Boysen/)

    await openCombobox('Rolle zuweisen')

    expect(screen.getByRole('option', { name: 'Fahrerin' })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: 'Einsatzleitung' })).not.toBeInTheDocument()
    expect(screen.queryByRole('option', { name: 'Gärtner' })).not.toBeInTheDocument()
  })

  it("offers roles from the caller's whole visible subtree in the list filter, unlike the assignment picker", async () => {
    render(<MembersPage />)

    await openCombobox('Rolle')

    // "Fahrerin" belongs to "nord", not the caller's own "amt" — proving the
    // filter is not limited to the caller's own organization.
    expect(screen.getByRole('option', { name: 'Fahrerin' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'Einsatzleitung' })).toBeInTheDocument()
  })

  it('revokes a role immediately', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await userEvent.click(screen.getByRole('button', { name: /Rolle Einsatzleitung entziehen/ }))

    expect(revokeMutate).toHaveBeenCalledWith(
      expect.objectContaining({ userId: 'u-anna', roleId: 'r-lead' }),
      expect.anything(),
    )
  })

  it('asks before moving a person to another organization', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await openCombobox('Organisation wechseln')
    await userEvent.click(screen.getByRole('option', { name: 'Stadtgärtnerei Nord' }))

    expect(setOrgMutate).not.toHaveBeenCalled()
    expect(screen.getByRole('alertdialog')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Verschieben' }))

    expect(setOrgMutate).toHaveBeenCalledWith(
      expect.objectContaining({ userId: 'u-anna', organizationId: 'nord' }),
    )
  })

  it('locks roles and organization on your own account', async () => {
    meId.mockReturnValue('u-anna')
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    expect(
      screen.queryByRole('button', { name: /Rolle Einsatzleitung entziehen/ }),
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole('combobox', { name: 'Organisation wechseln' }),
    ).not.toBeInTheDocument()
    // The own profile stays editable.
    expect(screen.getByLabelText('Telefonnummer')).toBeEnabled()
  })

  it('explains why a person without an organization has no assignable roles', async () => {
    render(<MembersPage />)
    await select(/Cem Demir/)

    expect(screen.getByText(/Ohne Organisation/)).toBeInTheDocument()
    expect(screen.queryByRole('combobox', { name: 'Rolle zuweisen' })).not.toBeInTheDocument()
  })

  it('hides every write action without user:update', async () => {
    permissions.mockReturnValue(new Set(['user:read']))
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    expect(
      screen.queryByRole('button', { name: /Rolle Einsatzleitung entziehen/ }),
    ).not.toBeInTheDocument()
    expect(screen.queryByRole('combobox', { name: 'Rolle zuweisen' })).not.toBeInTheDocument()
    expect(
      screen.queryByRole('combobox', { name: 'Organisation wechseln' }),
    ).not.toBeInTheDocument()
  })

  it('saves the profile only after the save button', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await editEmployeeId('EMP-9')
    expect(updateMutate).not.toHaveBeenCalled()

    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    expect(updateMutate).toHaveBeenCalledWith(
      expect.objectContaining({ userId: 'u-anna', employeeId: 'EMP-9' }),
      expect.anything(),
    )
  })

  // PUT /users/{id} replaces the whole profile: an omitted avatarUrl wipes it.
  it('carries the current avatar through a profile save', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await editEmployeeId('EMP-9')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    expect(updateMutate).toHaveBeenCalledWith(
      expect.objectContaining({ userId: 'u-anna', avatarUrl: avatarOf('u-anna') }),
      expect.anything(),
    )
  })

  it('sends empty profile text fields as null', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    await userEvent.clear(screen.getByLabelText('Personalnummer'))
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    expect(updateMutate).toHaveBeenCalledWith(
      expect.objectContaining({ userId: 'u-anna', employeeId: null }),
      expect.anything(),
    )
  })

  it('narrows the list by search', async () => {
    render(<MembersPage />)

    await userEvent.type(screen.getByRole('searchbox', { name: /suchen/i }), 'Boysen')

    await waitFor(() =>
      expect(list().queryByRole('button', { name: /Anna Ahlmann/ })).not.toBeInTheDocument(),
    )
    expect(list().getByRole('button', { name: /Bo Boysen/ })).toBeInTheDocument()
  })

  it('offers a way back once the selected person falls out of the filter', async () => {
    render(<MembersPage />)
    await select(/Anna Ahlmann/)
    expect(screen.getByRole('heading', { name: 'Anna Ahlmann' })).toBeInTheDocument()

    await userEvent.type(screen.getByRole('searchbox', { name: /suchen/i }), 'Boysen')

    await waitFor(() =>
      expect(
        screen.getByText('Die ausgewählte Person passt nicht zu Suche und Filter.'),
      ).toBeInTheDocument(),
    )
    expect(screen.queryByRole('heading', { name: 'Anna Ahlmann' })).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Suche und Filter zurücksetzen' }))

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Anna Ahlmann' })).toBeInTheDocument(),
    )
  })

  it('shows the 403 of a too-powerful role at the roles card', async () => {
    assignError.mockReturnValue({ response: { status: 403 } })
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    expect(screen.getByRole('alert')).toHaveTextContent(/Rechte/)
  })

  it('shows the 409 of an own-account role change at the roles card, not a toast', async () => {
    assignError.mockReturnValue({ response: { status: 409 } })
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    expect(screen.getByRole('alert')).toHaveTextContent(
      'Am eigenen Konto lassen sich Rollen nicht ändern.',
    )
    expect(screen.queryByText('Die Rolle konnte nicht zugewiesen werden.')).not.toBeInTheDocument()
  })

  it('shows the 409 of an own-account organization change at the organization card, not a toast', async () => {
    setOrgError.mockReturnValue({ response: { status: 409 } })
    render(<MembersPage />)
    await select(/Anna Ahlmann/)

    expect(screen.getByRole('alert')).toHaveTextContent(
      'Die eigene Organisation lässt sich hier nicht ändern.',
    )
    expect(
      screen.queryByText('Die Organisation konnte nicht geändert werden.'),
    ).not.toBeInTheDocument()
  })

  it('renders the detail in a drawer on narrow screens', async () => {
    isDesktop.mockReturnValue(false)
    render(<MembersPage />)

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    await select(/Anna Ahlmann/)

    expect(await screen.findByRole('dialog')).toBeInTheDocument()
  })
})
