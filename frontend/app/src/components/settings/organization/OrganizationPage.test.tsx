import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor, within } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type {
  OrganizationDetailResponse,
  OrganizationResponse,
  UserResponse,
} from '@/api/backendApi'
import type {
  CreateOrganizationVariables,
  DeleteOrganizationVariables,
  UpdateOrganizationVariables,
} from '@/hooks/useOrganizationMutations'

// The instance root ('root') sits above the user's own organization ('amt') and
// must stay out of the tree. parentId is left off entirely rather than set to
// null, because the generated client turns JSON null into undefined.
const BASE_ORGS: OrganizationResponse[] = [
  { id: 'root', name: 'Stadt Flensburg', memberCount: 1 },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
  { id: 'juergensby', name: 'Team Jürgensby', memberCount: 5, parentId: 'nord' },
]

const BASE_DETAILS: Record<string, OrganizationDetailResponse> = {
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
  organization: BASE_ORGS.find((org) => org.id === orgId),
})

const users: UserResponse[] = [
  member('u-anna', 'Anna', 'Ahlmann', 'amt'),
  member('u-bo', 'Bo', 'Boysen', 'nord'),
]

// The create and delete mocks rewrite these, so every test starts from a copy.
let orgList: OrganizationResponse[] = []
let detailMap: Record<string, OrganizationDetailResponse> = {}

const permissions = vi.fn((): Permissions => UNRESTRICTED)
vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => permissions() }))

// jsdom's matchMedia mock always reports non-desktop, which would wrap the detail
// panel in a Drawer and leave both panes unqueryable. Default to the desktop
// two-pane layout; one test flips this to exercise the Drawer branch.
const isDesktop = vi.fn((): boolean => true)
vi.mock('@/hooks/useMediaQuery', () => ({ useMediaQuery: () => isDesktop() }))

const createMutate = vi.fn()
const updateMutate = vi.fn()
const deleteMutate = vi.fn()
const createError = vi.fn((): unknown => null)
const updateError = vi.fn((): unknown => null)
vi.mock('@/hooks/useOrganizationMutations', () => ({
  useOrganizationMutations: () => ({
    createOrganization: {
      mutate: createMutate,
      isPending: false,
      error: createError(),
      reset: vi.fn(),
    },
    updateOrganization: {
      mutate: updateMutate,
      isPending: false,
      error: updateError(),
      reset: vi.fn(),
    },
    deleteOrganization: { mutate: deleteMutate, isPending: false, error: null, reset: vi.fn() },
  }),
}))

const blockerStatus = vi.fn((): string => 'idle')
const blockerProceed = vi.fn()
const blockerReset = vi.fn()
vi.mock('@tanstack/react-router', () => ({
  useBlocker: () => ({ status: blockerStatus(), proceed: blockerProceed, reset: blockerReset }),
  Link: ({ children }: { children: ReactNode }) => <a href="#link">{children}</a>,
}))

const ownOrgId = vi.fn((): string => 'amt')
const listError = vi.fn((): unknown => null)
const detailError = vi.fn((): unknown => null)

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
            organization: BASE_ORGS.find((org) => org.id === ownOrgId()),
          },
          isLoading: false,
        }
      }
      if (scope === 'users') {
        // The page must ask the backend for one organization's members instead
        // of filtering a truncated page client-side.
        const { organizationId } = (second ?? {}) as { organizationId?: string }
        return {
          data: { data: users.filter((user) => user.organization?.id === organizationId) },
          isLoading: false,
        }
      }
      if (scope === 'organizations' && second === undefined) {
        return { data: orgList, isLoading: false, error: listError() }
      }
      if (scope === 'organizations') {
        return { data: detailMap[second as string], isLoading: false, error: detailError() }
      }
      return { data: undefined, isLoading: false }
    },
  }
})

const { default: OrganizationPage } = await import('./OrganizationPage')

const tree = () => within(screen.getByRole('list', { name: 'Organisationsstruktur' }))

const treeRow = (name: RegExp) => {
  const row = tree().getByRole('button', { name }).closest('li')
  if (!row) throw new Error(`tree row ${String(name)} not found`)
  return within(row)
}

const cardOf = (title: string) => {
  const section = screen.getByRole('heading', { name: title }).closest('section')
  if (!section) throw new Error(`card ${title} not found`)
  return within(section)
}

const selectNord = () =>
  userEvent.click(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ }))

describe('OrganizationPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    orgList = [...BASE_ORGS]
    detailMap = { ...BASE_DETAILS }
    permissions.mockReturnValue(UNRESTRICTED)
    isDesktop.mockReturnValue(true)
    ownOrgId.mockReturnValue('amt')
    blockerStatus.mockReturnValue('idle')
    createError.mockReturnValue(null)
    updateError.mockReturnValue(null)
    listError.mockReturnValue(null)
    detailError.mockReturnValue(null)
    updateMutate.mockReset()
    createMutate.mockImplementation(
      (
        vars: CreateOrganizationVariables,
        opts?: { onSuccess?: (created: OrganizationResponse) => void },
      ) => {
        const created: OrganizationResponse = {
          id: 'sued',
          name: vars.name,
          memberCount: 0,
          parentId: vars.parentId,
        }
        orgList = [...orgList, created]
        detailMap[created.id] = { ...created }
        opts?.onSuccess?.(created)
      },
    )
    deleteMutate.mockImplementation(
      (vars: DeleteOrganizationVariables, opts?: { onSuccess?: () => void }) => {
        orgList = orgList.filter((org) => org.id !== vars.orgId)
        opts?.onSuccess?.()
      },
    )
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

    // 4 own + 6 Duburg + 5 Jürgensby; the exact wording belongs to OrganizationTree.
    expect(tree().getByText(/^15 Personen/)).toBeInTheDocument()
    expect(tree().queryByText(/^4 Personen/)).not.toBeInTheDocument()
    // 3 own + the 15 below Stadtgärtnerei Nord
    expect(tree().getByText(/^18 Personen/)).toBeInTheDocument()
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
    await selectNord()

    await userEvent.type(screen.getByRole('textbox', { name: 'PLZ' }), '24939')

    const save = screen.getByRole('button', { name: 'Speichern' })
    expect(save).toBeDisabled()
    expect(screen.getByText('Straße fehlt')).toBeInTheDocument()
    expect(screen.getByText('Ort fehlt')).toBeInTheDocument()

    await userEvent.click(save)
    expect(updateMutate).not.toHaveBeenCalled()
  })

  it('saves name, address and contact person together', async () => {
    render(<OrganizationPage />)
    await selectNord()

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

  // The detail response carries the raw id even when the identity provider
  // cannot resolve it; saving anything else must not clear the reference.
  it('keeps an unresolved contact person id when saving another field', async () => {
    detailMap.nord = { ...BASE_DETAILS.nord, contactPersonId: 'u-bo' }
    render(<OrganizationPage />)
    await selectNord()

    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    expect(updateMutate.mock.calls[0][0] as UpdateOrganizationVariables).toEqual({
      orgId: 'nord',
      name: 'Stadtgärtnerei Nord Ost',
      address: null,
      contactPersonId: 'u-bo',
    })
  })

  it('sends no address at all when all three address fields are empty', async () => {
    render(<OrganizationPage />)
    await selectNord()

    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    expect(updateMutate.mock.calls[0][0] as UpdateOrganizationVariables).toEqual({
      orgId: 'nord',
      name: 'Stadtgärtnerei Nord Ost',
      address: null,
      contactPersonId: null,
    })
  })

  it('reports a failed list load without claiming the organization is missing', () => {
    listError.mockReturnValue({ response: { status: 403 } })
    render(<OrganizationPage />)

    expect(screen.getByRole('alert')).toHaveTextContent(/Du darfst diese Organisation nicht/)
    expect(screen.queryByText(/wurde nicht gefunden/)).not.toBeInTheDocument()
  })

  it('reports a failed detail load instead of leaving the pane blank', () => {
    detailError.mockReturnValue({ response: { status: 500 } })
    render(<OrganizationPage />)

    expect(screen.getByRole('alert')).toHaveTextContent(/konnte nicht geladen werden/)
    // The tree still renders from the list query.
    expect(tree().getByRole('button', { name: /Grünflächenamt/ })).toBeInTheDocument()
  })

  it('associates the empty-name hint with the name field', async () => {
    render(<OrganizationPage />)
    await selectNord()

    const name = screen.getByRole('textbox', { name: 'Name' })
    await userEvent.clear(name)

    expect(name).toHaveAttribute('aria-invalid', 'true')
    expect(name).toHaveAccessibleDescription('Gib der Organisation einen Namen.')
  })

  it('renders the own organization read-only when it is the instance root', () => {
    ownOrgId.mockReturnValue('root')
    render(<OrganizationPage />)

    expect(
      screen.getByText('Die oberste Organisation dieser Instanz kann nicht bearbeitet werden.'),
    ).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Name' })).not.toBeInTheDocument()
    // Permissions are unrestricted here, so a missing delete action can only come
    // from the organization being the instance root.
    expect(screen.queryByRole('button', { name: /Organisation löschen/ })).not.toBeInTheDocument()
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
    await selectNord()

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

  it('shows only members of the selected organization in the avatar stack', async () => {
    render(<OrganizationPage />)

    expect(cardOf('Zugewiesene Mitarbeitende').getByText('AA')).toBeInTheDocument()
    expect(cardOf('Zugewiesene Mitarbeitende').queryByText('BB')).not.toBeInTheDocument()

    await selectNord()

    expect(cardOf('Zugewiesene Mitarbeitende').getByText('BB')).toBeInTheDocument()
    expect(cardOf('Zugewiesene Mitarbeitende').queryByText('AA')).not.toBeInTheDocument()
  })

  it('routes a 409 name conflict to the name field', () => {
    updateError.mockReturnValue({ response: { status: 409 } })
    render(<OrganizationPage />)

    expect(screen.getByRole('textbox', { name: 'Name' })).toHaveAccessibleDescription(
      'Eine Organisation mit diesem Namen existiert bereits.',
    )
    expect(cardOf('Kontaktperson').queryByText(/nicht zugeordnet/)).not.toBeInTheDocument()
    // The mutations hook owns generic failures; the page must not repeat them.
    expect(screen.queryByText(/konnte nicht gespeichert werden/)).not.toBeInTheDocument()
  })

  it('routes a 422 to the contact person control', () => {
    updateError.mockReturnValue({ response: { status: 422 } })
    render(<OrganizationPage />)

    expect(
      cardOf('Kontaktperson').getByText(
        'Diese Person ist dieser Organisation nicht zugeordnet und kann nicht Kontaktperson sein.',
      ),
    ).toBeInTheDocument()
    expect(screen.getByRole('textbox', { name: 'Name' })).not.toHaveAccessibleDescription()
    expect(screen.queryByText(/konnte nicht gespeichert werden/)).not.toBeInTheDocument()
  })

  it('selects the new sub-organization and expands its parent after creating', async () => {
    render(<OrganizationPage />)
    await selectNord()
    expect(tree().queryByRole('button', { name: /Team Duburg/ })).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: /Unterorganisation anlegen/ }))
    await userEvent.type(
      screen.getByRole('textbox', { name: 'Name der Organisation' }),
      'Stadtgärtnerei Süd',
    )
    await userEvent.click(screen.getByRole('button', { name: 'Anlegen' }))

    await waitFor(() => expect(createMutate).toHaveBeenCalled())
    expect(createMutate.mock.calls[0][0] as CreateOrganizationVariables).toEqual({
      parentId: 'nord',
      name: 'Stadtgärtnerei Süd',
    })

    // The dialog closes only on success.
    await waitFor(() =>
      expect(
        screen.queryByRole('textbox', { name: 'Name der Organisation' }),
      ).not.toBeInTheDocument(),
    )
    // The parent was expanded, which also reveals its other children.
    expect(tree().getByRole('button', { name: /Team Duburg/ })).toBeInTheDocument()
    expect(tree().getByRole('button', { name: /Stadtgärtnerei Süd/ })).toHaveAttribute(
      'aria-current',
      'true',
    )
  })

  it('keeps the create dialog open with the typed name on a conflict', async () => {
    createError.mockReturnValue({ response: { status: 409 } })
    // A conflict means no onSuccess, so the dialog is never told to close.
    createMutate.mockReset()
    render(<OrganizationPage />)

    await userEvent.click(screen.getByRole('button', { name: /Unterorganisation anlegen/ }))
    const nameInput = screen.getByRole('textbox', { name: 'Name der Organisation' })
    await userEvent.type(nameInput, 'Stadtgärtnerei Nord')
    await userEvent.click(screen.getByRole('button', { name: 'Anlegen' }))

    expect(
      screen.getByText('Eine Organisation mit diesem Namen existiert bereits.'),
    ).toBeInTheDocument()
    expect(nameInput).toHaveValue('Stadtgärtnerei Nord')
  })

  it('deletes an organization after confirmation and falls back to its parent', async () => {
    render(<OrganizationPage />)
    await selectNord()

    await userEvent.click(screen.getByRole('button', { name: /Organisation löschen/ }))
    expect(await screen.findByText('Organisation löschen?')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))

    await waitFor(() => expect(deleteMutate).toHaveBeenCalled())
    expect(deleteMutate.mock.calls[0][0] as DeleteOrganizationVariables).toEqual({ orgId: 'nord' })
    await waitFor(() =>
      expect(tree().queryByRole('button', { name: /Stadtgärtnerei Nord/ })).not.toBeInTheDocument(),
    )
    expect(tree().getByRole('button', { name: /Grünflächenamt/ })).toHaveAttribute(
      'aria-current',
      'true',
    )
  })

  it('closes the delete dialog when the deletion is rejected', async () => {
    deleteMutate.mockImplementation(
      (_vars: DeleteOrganizationVariables, opts?: { onError?: (error: unknown) => void }) => {
        opts?.onError?.({ response: { status: 409 } })
      },
    )
    render(<OrganizationPage />)
    await selectNord()

    await userEvent.click(screen.getByRole('button', { name: /Organisation löschen/ }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))

    // The explaining toast lives behind the dialog, so the dialog must go.
    await waitFor(() => expect(screen.queryByText('Organisation löschen?')).not.toBeInTheDocument())
    expect(tree().getByRole('button', { name: /Stadtgärtnerei Nord/ })).toBeInTheDocument()
  })

  it('asks before discarding unsaved changes when selecting another node', async () => {
    render(<OrganizationPage />)
    await selectNord()
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

  it('asks before discarding unsaved changes when opening the create dialog', async () => {
    render(<OrganizationPage />)
    await selectNord()
    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), ' Ost')

    await userEvent.click(screen.getByRole('button', { name: /Unterorganisation anlegen/ }))

    expect(await screen.findByText('Änderungen verwerfen?')).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Name der Organisation' })).not.toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Verwerfen' }))

    expect(
      await screen.findByRole('textbox', { name: 'Name der Organisation' }),
    ).toBeInTheDocument()
    expect(screen.queryByDisplayValue('Stadtgärtnerei Nord Ost')).not.toBeInTheDocument()
    expect(screen.getByDisplayValue('Stadtgärtnerei Nord')).toBeInTheDocument()
  })

  it('warns before leaving the page with unsaved changes', async () => {
    blockerStatus.mockReturnValue('blocked')
    render(<OrganizationPage />)

    expect(screen.getByText('Seite verlassen?')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Verlassen' }))
    expect(blockerProceed).toHaveBeenCalled()
  })

  it('renders the detail in a drawer on mobile', async () => {
    isDesktop.mockReturnValue(false)
    render(<OrganizationPage />)

    // Nothing is auto-opened; the tree is what the user sees first.
    expect(screen.queryByRole('textbox', { name: 'Name' })).not.toBeInTheDocument()

    await selectNord()

    const drawer = await screen.findByRole('dialog')
    expect(within(drawer).getByRole('textbox', { name: 'Name' })).toHaveValue('Stadtgärtnerei Nord')
  })
})
