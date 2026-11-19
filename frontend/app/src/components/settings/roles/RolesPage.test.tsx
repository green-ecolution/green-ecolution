import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type { Role } from '@/api/backendApi'
import type { CreateRoleVariables } from '@/hooks/useRoleMutations'
import type { ModuleManifest } from '@/modules/types'
import type { Resource } from '@/lib/auth/permissions'
import { getI18n } from '@/lib/i18n'
import { irrigationManifest } from '@green-ecolution/module-irrigation'

const templates: Role[] = [
  {
    id: 't1',
    name: 'Routenplanung',
    description: 'Planung von Bewässerungsrouten',
    isTemplate: true,
    templateKey: 'route_planning',
    permissions: ['watering_plan:read', 'tree:read'],
  },
]

const orgRoles: Role[] = [
  {
    id: 'c1',
    name: 'Routenplanung',
    description: 'Planung von Bewässerungsrouten',
    isTemplate: false,
    templateKey: 'route_planning',
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

const probeManifest = (key: string): ModuleManifest => ({
  key,
  labelKey: `${key}:label` as ModuleManifest['labelKey'],
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
  resources: [{ key: `${key}_item` as Resource, area: 'operations', textsKey: `${key}:item` }],
})
const probes = vi.hoisted(() => ({ list: [] as ModuleManifest[] }))
vi.mock('@/modules/registry', async () => {
  const { irrigationManifest: irrigation } = await import('@green-ecolution/module-irrigation')
  return {
    get moduleManifests() {
      return [irrigation, ...probes.list]
    },
  }
})

const selectedOrg = vi.hoisted(() => ({ id: 'nord' }))
vi.mock('../organization/useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: selectedOrg.id,
    organizations: [],
    perms: 'unrestricted',
    can: (required: readonly string[]) => {
      const held = grantedPermissions()
      return held === 'unrestricted' || required.length === 0 || required.some((p) => held.has(p))
    },
  }),
}))

const permissions = vi.fn((): Permissions => UNRESTRICTED)
const grantedPermissions = () => permissions()
vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => permissions() }))

// jsdom never runs layout (getBoundingClientRect always reports 0 width), so an
// unmocked useContainerWiderThan would report `false` forever and wrap the detail
// panel in a Drawer, hiding the list behind it (aria-hidden). Default to the
// desktop two-pane layout so both panes stay queryable; a single test flips this
// to false to exercise the mobile Drawer branch.
const isWide = vi.fn((): boolean => true)
vi.mock('@/hooks/useContainerWiderThan', () => ({
  useContainerWiderThan: () => ({ ref: vi.fn(), isWide: isWide() }),
}))

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

const activeByOrg = vi.hoisted((): { value: Record<string, string[]> } => ({ value: {} }))
const requestedOrgRoles = vi.hoisted(() => ({ ids: new Set<string>() }))

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[] }) => {
      const [scope, kind, id] = options.queryKey
      if (scope === 'modules') {
        return {
          data: {
            modules: [],
            organizations: Object.entries(activeByOrg.value).map(([organizationId, active]) => ({
              organizationId,
              active,
            })),
          },
          isLoading: false,
        }
      }
      if (scope === 'roles' && kind === 'templates') return { data: templates, isLoading: false }
      if (scope === 'roles' && kind === 'org') {
        requestedOrgRoles.ids.add(String(id))
        return { data: orgRoles, isLoading: false }
      }
      return { data: { data: [] }, isLoading: false }
    },
  }
})

const activeModulesByOrg = (value: Record<string, string[]>) => {
  activeByOrg.value = value
}
const orgRolesRequestedFor = () => [...requestedOrgRoles.ids]

const { default: RolesPage } = await import('./RolesPage')

const renderPage = () => render(<RolesPage />)

const createRoleThroughTheForm = async (name: string) => {
  await userEvent.click(screen.getByRole('button', { name: 'Neu' }))
  await userEvent.type(screen.getByRole('textbox', { name: 'Name der Rolle' }), name)
  await userEvent.click(screen.getByRole('button', { name: 'Rolle anlegen' }))
  await waitFor(() => expect(createMutate).toHaveBeenCalled())
}

describe('RolesPage', () => {
  beforeAll(() => {
    for (const [key, label] of [
      ['alpha', 'Probe Alpha'],
      ['beta', 'Probe Beta'],
    ]) {
      getI18n().addResourceBundle('de', key, {
        item: {
          label,
          singular: label,
          pluralNoun: label,
          description: 'Probe',
          hint: { read: 'r', create: 'c', update: 'u', delete: 'd' },
        },
      })
    }
  })

  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockReturnValue(UNRESTRICTED)
    isWide.mockReturnValue(true)
    selectedOrg.id = 'nord'
    requestedOrgRoles.ids.clear()
    probes.list = [probeManifest('alpha'), probeManifest('beta')]
    activeModulesByOrg({ nord: [irrigationManifest.key] })
  })

  it('lists the roles of the selected organization, not of the own one', () => {
    renderPage()
    expect(orgRolesRequestedFor()).toEqual(['nord'])
  })

  it('creates a new role in the selected organization', async () => {
    renderPage()
    await createRoleThroughTheForm('Gießtrupp')
    expect(createMutate).toHaveBeenCalledWith(
      expect.objectContaining({ orgId: 'nord', name: 'Gießtrupp' }),
      expect.anything(),
    )
  })

  it('shows only the modules active in the selected organization in the matrix', () => {
    activeModulesByOrg({ nord: ['alpha'], amt: ['alpha', 'beta'] })
    renderPage()
    expect(screen.getByText('Probe Alpha')).toBeInTheDocument()
    expect(screen.queryByText('Probe Beta')).not.toBeInTheDocument()
  })

  it('lists the system role and hides the untouched copy of it', () => {
    renderPage()
    expect(screen.getByText('System · nicht editierbar')).toBeInTheDocument()
    expect(screen.getByText('Bezirksleiter Nord')).toBeInTheDocument()
    // 'Routenplanung' exists once — as the template, not as its pristine copy
    expect(screen.getAllByText('Routenplanung')).toHaveLength(1)
  })

  it('shows the system role read-only when selected', async () => {
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: /^Routenplanung/ }))
    expect(screen.getByText(/Systemrollen sind schreibgeschützt/)).toBeInTheDocument()
  })

  it('prefills a copy and creates it with the full permission set', async () => {
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: /^Routenplanung/ }))
    await userEvent.click(screen.getByRole('button', { name: 'Kopieren & bearbeiten' }))

    expect(screen.getByRole('textbox', { name: 'Name der Rolle' })).toHaveValue(
      'Routenplanung (Kopie)',
    )

    await userEvent.click(screen.getByRole('button', { name: 'Rolle anlegen' }))
    await waitFor(() => expect(createMutate).toHaveBeenCalled())
    const createVars = createMutate.mock.calls[0][0] as CreateRoleVariables
    expect(createVars).toMatchObject({
      orgId: 'nord',
      name: 'Routenplanung (Kopie)',
      description: 'Kopie von Routenplanung',
    })
    expect([...createVars.permissions].sort()).toEqual(['tree:read', 'watering_plan:read'])
  })

  it('saves an edited own role', async () => {
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: /Bezirksleiter Nord/ }))
    await userEvent.type(screen.getByRole('textbox', { name: 'Name der Rolle' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    expect(updateMutate.mock.calls[0][0]).toMatchObject({ roleId: 'r1' })
  })

  it('hides the rows of an inactive module and still saves its stored permissions', async () => {
    activeModulesByOrg({ nord: [] })
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: /Bezirksleiter Nord/ }))

    expect(screen.getByText('Fahrzeuge')).toBeInTheDocument()
    expect(screen.queryByText('Einsatzpläne')).not.toBeInTheDocument()
    expect(screen.queryByText('Bäume')).not.toBeInTheDocument()

    await userEvent.type(screen.getByRole('textbox', { name: 'Name der Rolle' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    const vars = updateMutate.mock.calls[0][0] as { permissions: string[] }
    expect([...vars.permissions]).toEqual(['watering_plan:read'])
  })

  it('hides the create button without role:create', () => {
    permissions.mockReturnValue(new Set(['role:read']))
    renderPage()
    expect(screen.queryByRole('button', { name: 'Neu' })).not.toBeInTheDocument()
  })

  it('does not open the detail drawer on load on mobile', () => {
    permissions.mockReturnValue(UNRESTRICTED)
    isWide.mockReturnValue(false)
    renderPage()

    // The list is shown, but nothing is auto-selected into the drawer.
    expect(screen.getByText('System · nicht editierbar')).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Name der Rolle' })).not.toBeInTheDocument()
  })

  it('guards the mobile drawer close when the draft is dirty', async () => {
    permissions.mockReturnValue(UNRESTRICTED)
    isWide.mockReturnValue(false)
    renderPage()

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
