import { useState } from 'react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import type {
  OrganizationModuleResponse,
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

const modulesOf = (
  rows: Partial<OrganizationModuleResponse>[] = [{}],
): OrganizationModulesResponse =>
  ({
    modules: rows.map((row) => ({
      key: 'probe',
      enabled: true,
      inheritedEnabled: true,
      origin: 'default',
      fixedByAncestor: false,
      ...row,
    })),
  })

const settingsOf = (): OrganizationSettingsResponse =>
  ({
    sensorOfflineAfterSecs: { value: 86_400, inheritedValue: 86_400 },
    defectStreak: { value: 3, inheritedValue: 3 },
    mapView: {
      value: { center: [54.7923, 9.4358] },
      inheritedValue: { center: [54.7923, 9.4358] },
    },
  })

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

const calls: unknown[][] = []
const probeSave = vi.fn((_orgId: string, _value: string) => {
  calls.push(['probe-values'])
  return Promise.resolve()
})
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
  icon: () => null,
  descriptionKey: 'irrigation:label',
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
  settingsSection: defineSettingsSection<ProbeDraft>({
    useDraft: useProbeDraft,
    Section: ({ draft, locked }) => (
      <div>
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
const updateModulesMutate = vi.fn((vars: UpdateOrganizationModulesVariables) => {
  calls.push(['modules', vars.modules])
  return Promise.resolve()
})
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

describe('OrganizationAreasPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    calls.length = 0
    context.orgId = 'nord'
    context.perms = 'unrestricted'
    settingsMap = Object.fromEntries(BASE_ORGS.map((org) => [org.id, settingsOf()]))
    modulesMap = Object.fromEntries(BASE_ORGS.map((org) => [org.id, modulesOf()]))
    blocker.status = 'idle'
    blocker.shouldBlockFn = undefined
    settingsError.mockReturnValue(null)
    modulesError.mockReturnValue(null)
    probeLoadError.mockReturnValue(null)
  })

  const renderLoaded = async () => {
    const view = renderPage()
    await screen.findByRole('heading', { name: 'Aufgabenbereiche' })
    return view
  }

  it('names the organization in the intro', async () => {
    context.orgId = 'amt'
    await renderLoaded()
    expect(
      screen.getByText(/Schalte ein, was zu den Aufgaben von Grünflächenamt gehört/),
    ).toBeInTheDocument()
  })

  it('shows a card per module and the sensors card', async () => {
    await renderLoaded()
    expect(await screen.findByRole('switch', { name: 'Bewässerung' })).toBeChecked()
    expect(screen.getByText('Eingeschaltet')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Sensoren' })).toBeInTheDocument()
    expect(screen.getByLabelText('Sensor gilt als offline nach')).toHaveValue(24)
  })

  it('carries no inheritance vocabulary', async () => {
    await renderLoaded()
    await screen.findByRole('switch', { name: 'Bewässerung' })
    expect(
      screen.queryByText(
        /erben|Eigenen Wert|Untereinheiten|Zuletzt geändert|Geerbt|Vorgabe der Instanz/,
      ),
    ).toBeNull()
  })

  it('shows a module\u2019s fields as soon as it is switched on in the draft', async () => {
    modulesMap.nord = modulesOf([{ enabled: false, inheritedEnabled: false }])
    await renderLoaded()
    expect(screen.queryByLabelText('Probewert')).toBeNull()
    await userEvent.click(await screen.findByRole('switch', { name: 'Bewässerung' }))
    expect(await screen.findByLabelText('Probewert')).toBeInTheDocument()
  })

  it('saves in three phases: switch on, values, switch off', async () => {
    modulesMap.nord = modulesOf([
      { key: 'probe', enabled: false, inheritedEnabled: false },
      { key: 'other', enabled: true },
    ])
    await renderLoaded()
    await userEvent.click(await screen.findByRole('switch', { name: 'Bewässerung' }))
    await userEvent.click(screen.getByRole('switch', { name: 'other' }))
    await userEvent.type(await screen.findByLabelText('Probewert'), 'x')
    await userEvent.click(screen.getByRole('button', { name: 'Speichern' }))
    await waitFor(() => expect(calls).toHaveLength(3))
    expect(calls).toEqual([
      ['modules', { probe: true }],
      ['probe-values'],
      ['modules', { other: false }],
    ])
  })

  it('disables a switch a parent set and says who', async () => {
    modulesMap.nord = modulesOf([{ fixedByAncestor: true, setBy: { id: 'amt' } }])
    await renderLoaded()
    expect(await screen.findByRole('switch', { name: 'Bewässerung' })).toBeDisabled()
    expect(
      screen.getByText('Diesen Aufgabenbereich legt Grünflächenamt fest.'),
    ).toBeInTheDocument()
  })

  it('keeps save disabled while a value is invalid and explains it at the field', async () => {
    await renderLoaded()
    const field = await screen.findByLabelText('Sensor gilt als auffällig nach')
    await userEvent.clear(field)
    await userEvent.type(field, '2.5')
    expect(screen.getByRole('alert')).toHaveTextContent('Die Anzahl muss eine ganze Zahl sein.')
    expect(screen.getByRole('button', { name: 'Speichern' })).toBeDisabled()
  })

  it('is read-only without setting:update', async () => {
    context.perms = new Set(['setting:read'])
    await renderLoaded()
    expect(await screen.findByRole('switch', { name: 'Bewässerung' })).toBeDisabled()
    expect(screen.getByLabelText('Sensor gilt als offline nach')).toBeDisabled()
    expect(screen.queryByRole('button', { name: 'Speichern' })).toBeNull()
    expect(
      screen.getByText('Du kannst die Aufgabenbereiche ansehen, aber nicht ändern.'),
    ).toBeInTheDocument()
  })

  // An invalid draft builds no request, so `dirty` alone would let the typed
  // value disappear on the way out.
  it('still asks before leaving with an invalid module value typed', async () => {
    await renderLoaded()
    expect(blocker.shouldBlockFn?.()).toBe(false)
    await userEvent.type(await screen.findByLabelText('Probewert'), 'invalid')
    expect(blocker.shouldBlockFn?.()).toBe(true)
  })

  it('reports a failed settings load instead of hiding the cards', async () => {
    settingsError.mockReturnValue({ response: { status: 403 } })
    await renderLoaded()
    expect(screen.getByRole('alert')).toHaveTextContent(/Aufgabenbereiche dieser Organisation nicht/)
  })

  it('reports a failed module switch load instead of hiding the cards', async () => {
    modulesError.mockReturnValue({ response: { status: 500 } })
    await renderLoaded()
    expect(screen.getByRole('alert')).toHaveTextContent(/konnten nicht geladen werden/)
  })
})
