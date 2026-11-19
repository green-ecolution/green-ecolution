import { useState } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor, within } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import type {
  OrganizationModulesResponse,
  OrganizationResponse,
  OrganizationSettingsResponse,
} from '@/api/backendApi'
import type {
  UpdateOrganizationModulesVariables,
  UpdateSettingsVariables,
} from '@/hooks/useSettingsMutations'
import { defineSettingsSection } from '@/modules/settings'
import type { ModuleManifest, ModuleSettingsDraft } from '@/modules/types'

const BASE_ORGS: OrganizationResponse[] = [
  { id: 'root', name: 'Stadt Flensburg', memberCount: 1 },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
]

const modulesOf = (): OrganizationModulesResponse =>
  ({
    enforcedBy: null,
    modules: [
      {
        key: 'probe',
        enabled: true,
        inheritedEnabled: true,
        origin: 'inherited',
        source: { id: 'root' },
      },
    ],
  }) as unknown as OrganizationModulesResponse

const settingsOf = (): OrganizationSettingsResponse =>
  ({
    mapView: {
      value: { center: [54.7923, 9.4358], bbox: [54.7148, 9.2858, 54.8601, 9.5838] },
      origin: 'default',
      lastChange: null,
    },
    descendantsMayOverride: true,
    enforcedBy: null,
  }) as unknown as OrganizationSettingsResponse

let settingsMap: Record<string, OrganizationSettingsResponse> = {}
let modulesMap: Record<string, OrganizationModulesResponse> = {}

const context = vi.hoisted((): { orgId: string; perms: Permissions } => ({
  orgId: 'nord',
  perms: 'unrestricted',
}))

vi.mock('./useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: context.orgId,
    organizations: BASE_ORGS,
    perms: context.perms,
    can: (required: readonly string[]) =>
      context.perms === 'unrestricted' ||
      required.length === 0 ||
      required.some((p) => (context.perms as Set<string>).has(p)),
  }),
}))

// A module whose section is one text field; 'invalid' stands for a value its
// validator refuses, so the page's dirty/valid bookkeeping can be watched.
interface ProbeDraft extends ModuleSettingsDraft {
  value: string
  setValue: (value: string) => void
}

const probeSave = vi.fn((_orgId: string, _value: string) => Promise.resolve())
const probeLoadError = vi.fn((): unknown => null)

// Like a real module draft, it starts over for another organization.
const useProbeDraft = (organizationId: string, { enabled }: { enabled: boolean }): ProbeDraft => {
  const [edit, setEdit] = useState({ organizationId, value: '' })
  const value = edit.organizationId === organizationId ? edit.value : ''
  const valid = value !== 'invalid'
  return {
    loading: !enabled,
    loadError: probeLoadError(),
    dirty: valid && value !== '',
    valid,
    reset: () => setEdit({ organizationId, value: '' }),
    save: () => probeSave(organizationId, value),
    value,
    setValue: (next) => setEdit({ organizationId, value: next }),
  }
}

const probeManifest: ModuleManifest = {
  key: 'probe',
  labelKey: 'irrigation:label',
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
  settingsSection: defineSettingsSection<ProbeDraft>({
    useDraft: useProbeDraft,
    Section: ({ draft, locked, organizationName }) => (
      <div>
        <p>{`Probe-Vorgabe von ${organizationName('root') ?? '?'}`}</p>
        <label>
          Probewert
          <input
            value={draft.value}
            disabled={locked}
            onChange={(event) => draft.setValue(event.target.value)}
          />
        </label>
      </div>
    ),
  }),
}

vi.mock('@/modules/registry', () => ({ moduleManifests: [probeManifest] }))

const updateSettingsMutate = vi.fn((_vars: UpdateSettingsVariables) => Promise.resolve())
const updateModulesMutate = vi.fn((_vars: UpdateOrganizationModulesVariables) => Promise.resolve())
vi.mock('@/hooks/useSettingsMutations', async () => ({
  ...(await vi.importActual<typeof import('@/hooks/useSettingsMutations')>(
    '@/hooks/useSettingsMutations',
  )),
  useSettingsMutations: () => ({
    updateSettings: { mutateAsync: updateSettingsMutate, isPending: false },
  }),
  useUpdateOrganizationModules: () => ({ mutateAsync: updateModulesMutate, isPending: false }),
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const blocker = vi.hoisted(() => ({
  status: 'idle',
  shouldBlockFn: undefined as (() => boolean) | undefined,
}))
const blockerProceed = vi.fn()
const blockerReset = vi.fn()
vi.mock('@tanstack/react-router', () => ({
  useBlocker: (options: { shouldBlockFn: () => boolean }) => {
    blocker.shouldBlockFn = options.shouldBlockFn
    return { status: blocker.status, proceed: blockerProceed, reset: blockerReset }
  },
}))

const settingsError = vi.fn((): unknown => null)
const modulesError = vi.fn((): unknown => null)

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[]; enabled?: boolean }) => {
      const [scope, second, third] = options.queryKey
      if (scope === 'organizations' && third === 'modules') {
        const data = options.enabled === false ? undefined : modulesMap[second as string]
        return { data, isLoading: false, error: modulesError() }
      }
      if (scope === 'organizations' && third === 'settings') {
        const data = options.enabled === false ? undefined : settingsMap[second as string]
        return { data, isLoading: false, error: settingsError() }
      }
      return { data: undefined, isLoading: false }
    },
  }
})

const { default: OrganizationAreasPage } = await import('./OrganizationAreasPage')

const renderPage = () => render(<OrganizationAreasPage />)

const cardOf = (title: string) => {
  const section = screen.getByRole('heading', { name: title }).closest('section')
  if (!section) throw new Error(`card ${title} not found`)
  return within(section)
}

const overrideSwitch = () =>
  screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ })

describe('OrganizationAreasPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    context.orgId = 'nord'
    context.perms = 'unrestricted'
    settingsMap = Object.fromEntries(BASE_ORGS.map((org) => [org.id, settingsOf()]))
    modulesMap = Object.fromEntries(BASE_ORGS.map((org) => [org.id, modulesOf()]))
    blocker.status = 'idle'
    blocker.shouldBlockFn = undefined
    settingsError.mockReturnValue(null)
    modulesError.mockReturnValue(null)
    probeLoadError.mockReturnValue(null)
    probeSave.mockImplementation(() => Promise.resolve())
    updateSettingsMutate.mockImplementation(() => Promise.resolve())
    updateModulesMutate.mockImplementation(() => Promise.resolve())
  })

  it('names the organization in the intro', () => {
    context.orgId = 'amt'
    renderPage()
    expect(
      screen.getByText(/Schalte ein, was zu den Aufgaben von Grünflächenamt gehört/),
    ).toBeInTheDocument()
  })

  it('no longer shows the map view, which moved to the overview', () => {
    renderPage()
    expect(screen.queryByText('Kartenausschnitt')).not.toBeInTheDocument()
  })

  it('saves core settings, module switches and a module section with one button', async () => {
    renderPage()

    await userEvent.type(screen.getByRole('textbox', { name: 'Probewert' }), '48')
    await userEvent.click(overrideSwitch())
    await userEvent.click(cardOf('Module').getByRole('switch', { name: 'Bewässerung' }))
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateModulesMutate).toHaveBeenCalledTimes(1))
    expect(updateSettingsMutate).toHaveBeenCalledWith({
      orgId: 'nord',
      body: { descendantsMayOverride: false },
    })
    expect(probeSave).toHaveBeenCalledWith('nord', '48')
    expect(updateModulesMutate).toHaveBeenCalledWith({ orgId: 'nord', modules: { probe: false } })
    // One outcome for everything the button sent.
    await waitFor(() => expect(showToast).toHaveBeenCalledTimes(1))
    expect(showToast).toHaveBeenCalledWith('Einstellungen gespeichert')
  })

  it('saves only the switches when only a module switch changed', async () => {
    renderPage()

    expect(screen.queryByRole('button', { name: 'Speichern' })).not.toBeInTheDocument()
    await userEvent.click(cardOf('Module').getByRole('switch', { name: 'Bewässerung' }))
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateModulesMutate).toHaveBeenCalled())
    expect(updateModulesMutate.mock.calls[0][0]).toEqual({
      orgId: 'nord',
      modules: { probe: false },
    })
    expect(updateSettingsMutate).not.toHaveBeenCalled()
    expect(probeSave).not.toHaveBeenCalled()
  })

  it('shows a module section only where the module is active', () => {
    modulesMap.nord = {
      ...modulesOf(),
      modules: [
        { key: 'probe', enabled: false, inheritedEnabled: true, origin: 'own', ownValue: false },
      ],
    } as unknown as OrganizationModulesResponse
    renderPage()

    expect(screen.queryByRole('textbox', { name: 'Probewert' })).not.toBeInTheDocument()
    // The switch that turns it back on stays.
    expect(cardOf('Module').getByRole('switch', { name: 'Bewässerung' })).not.toBeChecked()
    expect(screen.getByRole('heading', { name: 'Fachliche Vorgaben' })).toBeInTheDocument()
  })

  it('reports a failed part with the error toast and keeps its draft', async () => {
    probeSave.mockImplementation(() => Promise.reject(new Error('boom')))
    // The core half goes through and comes back changed.
    updateSettingsMutate.mockImplementation(() => {
      settingsMap.nord = { ...settingsMap.nord, descendantsMayOverride: false }
      return Promise.resolve()
    })
    const { rerender } = renderPage()

    await userEvent.type(screen.getByRole('textbox', { name: 'Probewert' }), '48')
    await userEvent.click(overrideSwitch())
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith(
        'Die Einstellungen konnten nicht gespeichert werden.',
        'error',
      ),
    )
    expect(updateSettingsMutate).toHaveBeenCalled()
    rerender(<OrganizationAreasPage />)

    expect(screen.getByRole('textbox', { name: 'Probewert' })).toHaveValue('48')
    expect(overrideSwitch()).not.toBeChecked()
    expect(screen.getByRole('button', { name: 'Speichern' })).toBeInTheDocument()
  })

  it('keeps an invalid module edit when a valid core change is saved', async () => {
    const { rerender } = renderPage()

    await userEvent.type(screen.getByRole('textbox', { name: 'Probewert' }), 'invalid')
    await userEvent.click(overrideSwitch())
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(updateSettingsMutate).toHaveBeenCalled())
    // An invalid draft sends nothing.
    expect(probeSave).not.toHaveBeenCalled()
    settingsMap.nord = { ...settingsMap.nord, descendantsMayOverride: false }
    rerender(<OrganizationAreasPage />)

    await waitFor(() => expect(overrideSwitch()).not.toBeChecked())
    expect(screen.getByRole('textbox', { name: 'Probewert' })).toHaveValue('invalid')
  })

  // An invalid draft builds no request, so `dirty` alone would let the typed
  // value disappear on the way out.
  it('still asks before leaving with an invalid module value typed', async () => {
    renderPage()
    expect(blocker.shouldBlockFn?.()).toBe(false)

    await userEvent.type(screen.getByRole('textbox', { name: 'Probewert' }), 'invalid')

    expect(blocker.shouldBlockFn?.()).toBe(true)
  })

  it('discards a flipped module switch on cancel', async () => {
    renderPage()

    await userEvent.click(cardOf('Module').getByRole('switch', { name: 'Bewässerung' }))
    expect(cardOf('Module').getByRole('switch', { name: 'Bewässerung' })).not.toBeChecked()
    await userEvent.click(screen.getByRole('button', { name: 'Abbrechen' }))

    expect(cardOf('Module').getByRole('switch', { name: 'Bewässerung' })).toBeChecked()
  })

  it('reports a failed settings load instead of hiding the section', () => {
    settingsError.mockReturnValue({ response: { status: 403 } })
    renderPage()

    expect(screen.getByRole('alert')).toHaveTextContent(/Vorgaben dieser Organisation nicht/)
  })

  it('reports a failed module section load instead of hiding the section', () => {
    probeLoadError.mockReturnValue({ response: { status: 500 } })
    renderPage()

    expect(screen.getByRole('alert')).toBeInTheDocument()
    expect(screen.queryByRole('textbox', { name: 'Probewert' })).not.toBeInTheDocument()
  })

  it('reports a failed module switch load instead of hiding the section', () => {
    modulesError.mockReturnValue({ response: { status: 500 } })
    renderPage()

    expect(screen.getByRole('alert')).toHaveTextContent(/Vorgaben konnten nicht geladen werden/)
  })

  it('shows the settings read-only without setting:update', () => {
    context.perms = new Set(['setting:read'])
    renderPage()

    expect(screen.getByText('Fachliche Vorgaben')).toBeInTheDocument()
    expect(screen.getByRole('textbox', { name: 'Probewert' })).toBeDisabled()
    expect(overrideSwitch()).toBeDisabled()
    expect(cardOf('Module').getByRole('switch', { name: 'Bewässerung' })).toBeDisabled()
  })
})
