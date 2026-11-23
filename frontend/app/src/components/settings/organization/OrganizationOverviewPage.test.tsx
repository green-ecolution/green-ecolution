import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import type {
  OrganizationDetailResponse,
  OrganizationResponse,
  OrganizationSettingsResponse,
  UserResponse,
} from '@/api/backendApi'
import type {
  DeleteOrganizationVariables,
  UpdateOrganizationVariables,
} from '@/hooks/useOrganizationMutations'
import type { UpdateSettingsVariables } from '@/hooks/useSettingsMutations'
import { getI18n } from '@/lib/i18n'
import type { ModuleManifest } from '@/modules/types'

const BASE_ORGS: OrganizationResponse[] = [
  { id: 'root', name: 'Stadt Flensburg', memberCount: 1 },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
]

const BASE_DETAILS: Record<string, OrganizationDetailResponse> = {
  root: { id: 'root', name: 'Stadt Flensburg', memberCount: 1 },
  amt: { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  nord: { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  duburg: { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
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
  wateringPlanSelectable: true,
  roles: [],
  status: 'available',
  organization: BASE_ORGS.find((org) => org.id === orgId),
})

const users: UserResponse[] = [
  member('u-anna', 'Anna', 'Ahlmann', 'amt'),
  member('u-bo', 'Bo', 'Boysen', 'nord'),
]

const settingsOf = (): OrganizationSettingsResponse => ({
  sensorOfflineAfterSecs: { value: 3600, inheritedValue: 3600 },
  defectStreak: { value: 3, inheritedValue: 3 },
  mapView: {
    value: {
      center: [54.7923, 9.4358],
      bbox: [54.7148, 9.2858, 54.8601, 9.5838],
      minZoom: 13,
      maxZoom: 18,
    },
    inheritedValue: {
      center: [54.7923, 9.4358],
      bbox: [54.7148, 9.2858, 54.8601, 9.5838],
      minZoom: 13,
      maxZoom: 18,
    },
  },
})

let visibleOrganizations = BASE_ORGS
// null: every listed organization came from GET /organizations.
let readableIds: Set<string> | null = null
let detailMap: Record<string, OrganizationDetailResponse> = {}
let settingsMap: Record<string, OrganizationSettingsResponse> = {}

const context = vi.hoisted((): { orgId: string; perms: Permissions } => ({
  orgId: 'amt',
  perms: 'unrestricted',
}))

vi.mock('./useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: context.orgId,
    organizations: visibleOrganizations,
    readableIds: readableIds ?? new Set(visibleOrganizations.map((org) => org.id)),
    perms: context.perms,
    can: (required: readonly string[]) =>
      context.perms === 'unrestricted' ||
      required.length === 0 ||
      required.some((p) => (context.perms as Set<string>).has(p)),
  }),
}))

const probeManifest = (key: string): ModuleManifest => ({
  key,
  labelKey: `probe:${key}` as ModuleManifest['labelKey'],
  icon: () => null,
  descriptionKey: `probe:${key}` as ModuleManifest['labelKey'],
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
})

const registry = vi.hoisted(() => ({ manifests: [] as ModuleManifest[] }))
vi.mock('@/modules/registry', () => ({
  get moduleManifests() {
    return registry.manifests
  },
}))

let activeModules: string[] = []

const updateMutate = vi.fn()
const deleteMutate = vi.fn()
const updateError = vi.fn((): unknown => null)
vi.mock('@/hooks/useOrganizationMutations', () => ({
  useOrganizationMutations: () => ({
    createOrganization: { mutate: vi.fn(), isPending: false, error: null, reset: vi.fn() },
    updateOrganization: {
      mutate: updateMutate,
      isPending: false,
      error: updateError(),
      reset: vi.fn(),
    },
    deleteOrganization: { mutate: deleteMutate, isPending: false, error: null, reset: vi.fn() },
  }),
}))

const updateSettingsMutate = vi.fn((_vars: UpdateSettingsVariables) => Promise.resolve())
vi.mock('@/hooks/useSettingsMutations', async () => ({
  ...(await vi.importActual<typeof import('@/hooks/useSettingsMutations')>(
    '@/hooks/useSettingsMutations',
  )),
  useSettingsMutations: () => ({
    updateSettings: { mutateAsync: updateSettingsMutate, isPending: false },
  }),
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const navigate = vi.fn()
const blockerStatus = vi.fn((): string => 'idle')
const blockerProceed = vi.fn()
const blockerReset = vi.fn()
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
  useNavigate: () => navigate,
  useBlocker: () => ({ status: blockerStatus(), proceed: blockerProceed, reset: blockerReset }),
}))

const detailError = vi.fn((): unknown => null)
const settingsError = vi.fn((): unknown => null)

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[]; enabled?: boolean }) => {
      const [scope, second, third] = options.queryKey
      if (scope === 'users') {
        const { organizationId } = (second ?? {}) as { organizationId?: string }
        return {
          data: { data: users.filter((user) => user.organization?.id === organizationId) },
          isLoading: false,
        }
      }
      if (scope === 'modules') {
        return {
          data: { organizations: [{ organizationId: context.orgId, active: activeModules }] },
          isLoading: false,
        }
      }
      if (scope === 'organizations' && third === 'settings') {
        const data = options.enabled === false ? undefined : settingsMap[second as string]
        return { data, isLoading: false, error: settingsError() }
      }
      if (scope === 'organizations' && second !== undefined && third === undefined) {
        return { data: detailMap[second as string], isLoading: false, error: detailError() }
      }
      return { data: undefined, isLoading: false }
    },
  }
})

// MapLibre needs a WebGL context, which jsdom does not provide.
vi.mock('@/components/map-gl/MapPreview', () => ({
  default: () => <div data-testid="map-preview" />,
}))

const { default: OrganizationOverviewPage } = await import('./OrganizationOverviewPage')

const renderPage = () => render(<OrganizationOverviewPage />)

const editMapView = () =>
  userEvent.click(screen.getByRole('switch', { name: 'Keine Einschränkung' }))

describe('OrganizationOverviewPage', () => {
  beforeAll(() => {
    getI18n().addResourceBundle('de', 'probe', { alpha: 'Bewässerung', beta: 'Abfall' })
  })

  beforeEach(() => {
    vi.clearAllMocks()
    registry.manifests = []
    activeModules = []
    visibleOrganizations = BASE_ORGS
    readableIds = null
    context.orgId = 'amt'
    context.perms = 'unrestricted'
    detailMap = { ...BASE_DETAILS }
    settingsMap = Object.fromEntries(BASE_ORGS.map((org) => [org.id, settingsOf()]))
    blockerStatus.mockReturnValue('idle')
    updateError.mockReturnValue(null)
    detailError.mockReturnValue(null)
    settingsError.mockReturnValue(null)
    updateMutate.mockReset()
    updateSettingsMutate.mockImplementation(() => Promise.resolve())
    deleteMutate.mockImplementation(
      (_vars: DeleteOrganizationVariables, opts?: { onSuccess?: () => void }) => {
        opts?.onSuccess?.()
      },
    )
  })

  it('blocks saving while the address is partial', async () => {
    context.orgId = 'nord'
    renderPage()

    await userEvent.type(screen.getByRole('textbox', { name: 'PLZ' }), '24939')

    const save = screen.getByRole('button', { name: 'Speichern' })
    expect(save).toBeDisabled()
    expect(screen.getByText('Straße fehlt')).toBeInTheDocument()
    expect(screen.getByText('Ort fehlt')).toBeInTheDocument()

    await userEvent.click(save)
    expect(updateMutate).not.toHaveBeenCalled()
  })

  it('saves name, address and contact person together', async () => {
    context.orgId = 'nord'
    renderPage()

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

  it('keeps an unresolved contact person id when saving another field', async () => {
    detailMap.nord = { ...BASE_DETAILS.nord, contactPersonId: 'u-bo' }
    context.orgId = 'nord'
    renderPage()

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
    context.orgId = 'nord'
    renderPage()

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

  it('reports a failed detail load instead of leaving the pane blank', () => {
    detailError.mockReturnValue({ response: { status: 500 } })
    renderPage()

    expect(screen.getByRole('alert')).toHaveTextContent(/konnte nicht geladen werden/)
  })

  it('associates the empty-name hint with the name field', async () => {
    context.orgId = 'nord'
    renderPage()

    const name = screen.getByRole('textbox', { name: 'Name' })
    await userEvent.clear(name)

    expect(name).toHaveAttribute('aria-invalid', 'true')
    expect(name).toHaveAccessibleDescription('Gib der Organisation einen Namen.')
  })

  it('renders the own organization read-only when it is the instance root', () => {
    context.orgId = 'root'
    renderPage()

    expect(
      screen.getByText('Die oberste Organisation dieser Instanz kann nicht bearbeitet werden.'),
    ).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Name' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Organisation löschen/ })).not.toBeInTheDocument()
  })

  it('hides the delete action without organization:delete', () => {
    context.orgId = 'nord'
    context.perms = new Set(['organization:read', 'organization:update', 'user:read'])
    renderPage()

    expect(screen.getByRole('textbox', { name: 'Name' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /Organisation löschen/ })).not.toBeInTheDocument()
    expect(screen.queryByText(/Eine Organisation lässt sich nur löschen/)).not.toBeInTheDocument()
  })

  it('routes a 409 name conflict to the name field', () => {
    updateError.mockReturnValue({ response: { status: 409 } })
    renderPage()

    expect(screen.getByRole('textbox', { name: 'Name' })).toHaveAccessibleDescription(
      'Eine Organisation mit diesem Namen existiert bereits.',
    )
    expect(screen.queryByText(/nicht zugeordnet/)).not.toBeInTheDocument()
    expect(screen.queryByText(/konnte nicht gespeichert werden/)).not.toBeInTheDocument()
  })

  it('routes a 422 to the contact person control', () => {
    updateError.mockReturnValue({ response: { status: 422 } })
    renderPage()

    expect(
      screen.getByText(
        'Diese Person ist dieser Organisation nicht zugeordnet und kann nicht Kontaktperson sein.',
      ),
    ).toBeInTheDocument()
    expect(screen.getByRole('textbox', { name: 'Name' })).not.toHaveAccessibleDescription()
    expect(screen.queryByText(/konnte nicht gespeichert werden/)).not.toBeInTheDocument()
  })

  it('closes the delete dialog when the deletion is rejected', async () => {
    deleteMutate.mockImplementation(
      (_vars: DeleteOrganizationVariables, opts?: { onError?: (error: unknown) => void }) => {
        opts?.onError?.({ response: { status: 409 } })
      },
    )
    context.orgId = 'nord'
    renderPage()

    await userEvent.click(screen.getByRole('button', { name: /Organisation löschen/ }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))

    await waitFor(() => expect(screen.queryByText('Organisation löschen?')).not.toBeInTheDocument())
    expect(navigate).not.toHaveBeenCalled()
  })

  it('warns before leaving the page with unsaved changes', async () => {
    blockerStatus.mockReturnValue('blocked')
    renderPage()

    expect(screen.getByText('Seite verlassen?')).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: 'Verlassen' }))
    expect(blockerProceed).toHaveBeenCalled()
  })

  it('does not call the settings endpoints when only the name changed', async () => {
    context.orgId = 'nord'
    renderPage()

    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateMutate).toHaveBeenCalled())
    expect(updateSettingsMutate).not.toHaveBeenCalled()
  })

  it('leaves the organization alone when only a setting changed', async () => {
    context.orgId = 'nord'
    renderPage()

    await editMapView()
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateSettingsMutate).toHaveBeenCalled())
    expect(updateMutate).not.toHaveBeenCalled()
  })

  it('saves a changed setting although the master data is still incomplete', async () => {
    context.orgId = 'nord'
    renderPage()

    await userEvent.type(
      screen.getByRole('textbox', { name: 'Straße und Hausnummer' }),
      'Gartenweg 1',
    )
    await editMapView()
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateSettingsMutate).toHaveBeenCalled())
    expect(updateMutate).not.toHaveBeenCalled()
  })

  it('returns to the usual viewport and saves exactly that viewport', async () => {
    context.orgId = 'nord'
    const own = { ...settingsOf().mapView.value, center: [54.8, 9.44] }
    const settings = settingsOf()
    settingsMap.nord = { ...settings, mapView: { ...settings.mapView, value: own } }
    renderPage()

    await userEvent.click(screen.getByRole('button', { name: 'Üblichen Ausschnitt verwenden' }))
    expect(
      screen.queryByRole('button', { name: 'Üblichen Ausschnitt verwenden' }),
    ).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() =>
      expect(updateSettingsMutate).toHaveBeenCalledWith(
        expect.objectContaining({
          orgId: 'nord',
          body: { mapView: settings.mapView.inheritedValue },
        }),
      ),
    )
  })

  it('offers the usual viewport only while the draft differs and the viewer may update', async () => {
    context.orgId = 'nord'
    renderPage()
    expect(
      screen.queryByRole('button', { name: 'Üblichen Ausschnitt verwenden' }),
    ).not.toBeInTheDocument()

    await editMapView()
    expect(
      screen.getByRole('button', { name: 'Üblichen Ausschnitt verwenden' }),
    ).toBeInTheDocument()
  })

  it('offers no usual viewport without setting:update', () => {
    context.orgId = 'nord'
    context.perms = new Set(['organization:read', 'setting:read'])
    const settings = settingsOf()
    settingsMap.nord = {
      ...settings,
      mapView: { ...settings.mapView, value: { ...settings.mapView.value, center: [54.8, 9.44] } },
    }
    renderPage()
    expect(
      screen.queryByRole('button', { name: 'Üblichen Ausschnitt verwenden' }),
    ).not.toBeInTheDocument()
  })

  it('offers saving on the read-only root when a setting changed', async () => {
    context.orgId = 'root'
    renderPage()

    expect(screen.queryByRole('button', { name: 'Speichern' })).not.toBeInTheDocument()

    await editMapView()
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() =>
      expect(updateSettingsMutate).toHaveBeenCalledWith(expect.objectContaining({ orgId: 'root' })),
    )
    expect(updateMutate).not.toHaveBeenCalled()
  })

  it('shows no organization tree and no breadcrumb above the organization', () => {
    renderPage()
    expect(screen.queryByRole('tree')).not.toBeInTheDocument()
    expect(screen.queryByRole('navigation', { name: /breadcrumb/i })).not.toBeInTheDocument()
  })

  it('links to members, structure and areas of the same organization', () => {
    renderPage()
    expect(screen.getByRole('link', { name: /Mitarbeitende/ })).toHaveAttribute(
      'href',
      '/settings/organization/amt/members',
    )
    expect(screen.getByRole('link', { name: /Struktur/ })).toHaveAttribute(
      'href',
      '/settings/organization/amt/structure',
    )
    expect(screen.getByRole('link', { name: /Aufgabenbereiche/ })).toHaveAttribute(
      'href',
      '/settings/organization/amt/areas',
    )
  })

  it('hides the members link without user:read in this organization', () => {
    context.perms = new Set(['organization:read'])
    renderPage()
    expect(screen.queryByRole('link', { name: /Mitarbeitende/ })).not.toBeInTheDocument()
  })

  it('edits the map view directly with setting:update', () => {
    renderPage()
    expect(screen.getByRole('switch', { name: 'Keine Einschränkung' })).toBeEnabled()
    expect(
      screen.queryByRole('button', { name: /Wieder erben|Eigenen Wert/ }),
    ).not.toBeInTheDocument()
  })

  it('shows the map view as a preview only without setting:update', () => {
    context.perms = new Set(['organization:read', 'setting:read'])
    renderPage()
    expect(screen.getByRole('switch', { name: 'Keine Einschränkung' })).toBeDisabled()
    expect(screen.getByTestId('map-preview')).toBeInTheDocument()
  })

  it('hides the map view without setting:read in this organization', () => {
    context.perms = new Set(['organization:read'])
    renderPage()
    expect(screen.queryByText('Kartenausschnitt')).not.toBeInTheDocument()
  })

  it('opens the parent overview after deleting the organization', async () => {
    context.orgId = 'nord'
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: 'Organisation löschen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))
    expect(navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId',
      params: { orgId: 'amt' },
      ignoreBlocker: true,
    })
  })

  it('skips the leave-page blocker when deleting with unsaved edits', async () => {
    context.orgId = 'nord'
    renderPage()
    await userEvent.type(screen.getByRole('textbox', { name: 'Name' }), ' Ost')
    await userEvent.click(screen.getByRole('button', { name: 'Organisation löschen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))
    expect(navigate).toHaveBeenCalledWith(expect.objectContaining({ ignoreBlocker: true }))
  })

  // The deleted organization drops out of the context before onSuccess runs.
  it('opens the parent overview even when the list no longer holds the deleted one', async () => {
    let finishDelete: (() => void) | undefined
    deleteMutate.mockImplementation(
      (_vars: DeleteOrganizationVariables, opts?: { onSuccess?: () => void }) => {
        finishDelete = () => opts?.onSuccess?.()
      },
    )
    context.orgId = 'nord'
    const { rerender } = renderPage()
    await userEvent.click(screen.getByRole('button', { name: 'Organisation löschen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))

    visibleOrganizations = BASE_ORGS.filter((org) => org.id !== 'nord')
    rerender(<OrganizationOverviewPage />)
    finishDelete?.()

    expect(navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId',
      params: { orgId: 'amt' },
      ignoreBlocker: true,
    })
  })

  it('falls back to the entry when the parent is only reachable, not readable', async () => {
    context.orgId = 'nord'
    readableIds = new Set(['root', 'nord', 'duburg'])
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: 'Organisation löschen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))
    expect(navigate).toHaveBeenCalledWith({
      to: '/settings/organization',
      ignoreBlocker: true,
    })
  })

  it('falls back to the entry when the parent is not visible', async () => {
    context.orgId = 'nord'
    visibleOrganizations = BASE_ORGS.filter((org) => org.id !== 'amt')
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: 'Organisation löschen' }))
    await userEvent.click(screen.getByRole('button', { name: 'Löschen' }))
    expect(navigate).toHaveBeenCalledWith({
      to: '/settings/organization',
      ignoreBlocker: true,
    })
  })

  it('summarises the active modules on the areas link', () => {
    registry.manifests = [probeManifest('alpha'), probeManifest('beta')]
    activeModules = ['alpha', 'beta']
    renderPage()
    expect(screen.getByText('Bewässerung und Abfall sind eingeschaltet')).toBeInTheDocument()
  })

  it('uses the singular for a single active module', () => {
    registry.manifests = [probeManifest('alpha'), probeManifest('beta')]
    activeModules = ['alpha']
    renderPage()
    expect(screen.getByText('Bewässerung ist eingeschaltet')).toBeInTheDocument()
  })

  it('says so when no module is switched on', () => {
    registry.manifests = [probeManifest('alpha')]
    renderPage()
    expect(screen.getByText('Kein Aufgabenbereich ist eingeschaltet')).toBeInTheDocument()
  })
})
