import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type { Role } from '@/api/backendApi'
import type { CreateRoleVariables } from '@/hooks/useRoleMutations'

const templates: Role[] = [
  {
    id: 't1',
    name: 'Routenplanung',
    description: 'Planung von Bewässerungsrouten',
    isTemplate: true,
    permissions: ['watering_plan:read', 'tree:read'],
  },
]

const orgRoles: Role[] = [
  {
    id: 'c1',
    name: 'Routenplanung',
    description: 'Planung von Bewässerungsrouten',
    isTemplate: false,
    permissions: ['watering_plan:read', 'tree:read'],
  },
  {
    id: 'r1',
    name: 'Bezirksleiter Nord',
    description: 'Kopie von Routenplanung',
    isTemplate: false,
    permissions: ['watering_plan:read'],
  },
]

const permissions = vi.fn((): Permissions => UNRESTRICTED)
vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => permissions() }))

// jsdom's matchMedia mock always reports non-desktop, which would wrap the
// detail panel in a Drawer and hide the list behind it (aria-hidden). Default
// to the desktop two-pane layout so both panes stay queryable; a single test
// flips this to false to exercise the mobile Drawer branch.
const isDesktop = vi.fn((): boolean => true)
vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => isDesktop() }))

const createMutate = vi.fn()
const updateMutate = vi.fn()
const deleteMutate = vi.fn()
vi.mock('@/hooks/useRoleMutations', () => ({
  useRoleMutations: () => ({
    createRole: { mutate: createMutate, isPending: false, error: null, reset: vi.fn() },
    updateRole: { mutate: updateMutate, isPending: false, error: null, reset: vi.fn() },
    deleteRole: { mutate: deleteMutate, isPending: false, error: null, reset: vi.fn() },
  }),
}))

vi.mock('@tanstack/react-router', () => ({ useBlocker: () => ({ status: 'idle' }) }))

const me = {
  id: 'u1',
  organization: { id: 'org-1', name: 'Green Ecolution' },
  roles: [],
}

vi.mock('@tanstack/react-query', async () => {
  const actual = await vi.importActual<typeof import('@tanstack/react-query')>(
    '@tanstack/react-query',
  )
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[] }) => {
      const key = JSON.stringify(options.queryKey)
      if (key.includes('templates')) return { data: templates, isLoading: false }
      if (key.includes('org')) return { data: orgRoles, isLoading: false }
      if (key.includes('me')) return { data: me, isLoading: false }
      return { data: { data: [] }, isLoading: false }
    },
  }
})

const { default: RolesPage } = await import('./RolesPage')

describe('RolesPage', () => {
  beforeEach(() => vi.clearAllMocks())

  it('lists the system role and hides the untouched copy of it', () => {
    render(<RolesPage />)
    expect(screen.getByText('System · nicht editierbar')).toBeInTheDocument()
    expect(screen.getByText('Bezirksleiter Nord')).toBeInTheDocument()
    // 'Routenplanung' exists once — as the template, not as its pristine copy
    expect(screen.getAllByText('Routenplanung')).toHaveLength(1)
  })

  it('shows the system role read-only when selected', async () => {
    render(<RolesPage />)
    await userEvent.click(screen.getByRole('button', { name: /^Routenplanung/ }))
    expect(screen.getByText(/Systemrollen sind schreibgeschützt/)).toBeInTheDocument()
  })

  it('prefills a copy and creates it with the full permission set', async () => {
    render(<RolesPage />)
    await userEvent.click(screen.getByRole('button', { name: /^Routenplanung/ }))
    await userEvent.click(screen.getByRole('button', { name: 'Kopieren & bearbeiten' }))

    expect(screen.getByRole('textbox', { name: 'Name der Rolle' })).toHaveValue(
      'Routenplanung (Kopie)',
    )

    await userEvent.click(screen.getByRole('button', { name: 'Rolle anlegen' }))
    await waitFor(() => expect(createMutate).toHaveBeenCalled())
    const createVars = createMutate.mock.calls[0][0] as CreateRoleVariables
    expect(createVars).toMatchObject({
      orgId: 'org-1',
      name: 'Routenplanung (Kopie)',
      description: 'Kopie von Routenplanung',
    })
    expect([...createVars.permissions].sort()).toEqual(['tree:read', 'watering_plan:read'])
  })

  it('saves an edited own role', async () => {
    render(<RolesPage />)
    await userEvent.click(screen.getByRole('button', { name: /Bezirksleiter Nord/ }))
    await userEvent.type(screen.getByRole('textbox', { name: 'Name der Rolle' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    expect(updateMutate.mock.calls[0][0]).toMatchObject({ roleId: 'r1' })
  })

  it('hides the create button without role:create', () => {
    permissions.mockReturnValue(new Set(['role:read']))
    render(<RolesPage />)
    expect(screen.queryByRole('button', { name: 'Neu' })).not.toBeInTheDocument()
  })

  it('does not open the detail drawer on load on mobile', () => {
    permissions.mockReturnValue(UNRESTRICTED)
    isDesktop.mockReturnValue(false)
    render(<RolesPage />)

    // The list is shown, but nothing is auto-selected into the drawer.
    expect(screen.getByText('System · nicht editierbar')).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Name der Rolle' })).not.toBeInTheDocument()
  })

  it('guards the mobile drawer close when the draft is dirty', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    isDesktop.mockReturnValue(false)
    render(<RolesPage />)

    // Mobile shows the list first; tap a role to open the detail drawer.
    await userEvent.click(screen.getByRole('button', { name: /Bezirksleiter Nord/ }))

    const name = screen.getByRole('textbox', { name: 'Name der Rolle' })
    await userEvent.type(name, ' Ost')
    await userEvent.keyboard('{Escape}')

    // Dismissing a dirty drawer must ask before discarding, not drop edits.
    // The modal marks the drawer aria-hidden, so probe the still-mounted draft
    // by its value rather than its role.
    expect(await screen.findByText('Änderungen verwerfen?')).toBeInTheDocument()
    expect(screen.getByDisplayValue('Bezirksleiter Nord Ost')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Verwerfen' }))
    await waitFor(() =>
      expect(screen.queryByDisplayValue('Bezirksleiter Nord Ost')).not.toBeInTheDocument(),
    )
  })
})
