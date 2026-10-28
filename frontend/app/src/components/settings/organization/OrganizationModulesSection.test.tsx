import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { ModuleActivationDraft, ModuleActivationRow } from './useModuleActivationDraft'

vi.mock('@/modules/registry', () => ({
  moduleManifests: [
    {
      key: 'probe',
      labelKey: 'irrigation:label',
      i18n: { namespaces: [], load: () => Promise.resolve({}) },
    },
  ],
}))

const { default: OrganizationModulesSection } = await import('./OrganizationModulesSection')

const setOwn = vi.fn()

const ROWS: ModuleActivationRow[] = [
  {
    key: 'probe',
    enabled: true,
    origin: 'inherited',
    sourceId: 'root',
    own: null,
    setBy: null,
    locked: false,
  },
  {
    key: 'waste',
    enabled: false,
    origin: 'own',
    sourceId: null,
    own: false,
    setBy: null,
    locked: false,
  },
]

const draftOf = (overrides: Partial<ModuleActivationDraft> = {}): ModuleActivationDraft => ({
  loading: false,
  loadError: null,
  dirty: false,
  valid: true,
  reset: vi.fn(),
  save: vi.fn(),
  rows: ROWS,
  enforcedBy: null,
  isActive: () => true,
  setOwn,
  ...overrides,
})

const names: Record<string, string> = { root: 'Stadt Flensburg' }
const organizationName = (id: string | null | undefined) => (id ? (names[id] ?? null) : null)

describe('OrganizationModulesSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('lists every registered module under its manifest label', () => {
    render(
      <OrganizationModulesSection
        draft={draftOf()}
        canUpdate
        organizationName={organizationName}
      />,
    )

    expect(screen.getByRole('heading', { name: 'Module' })).toBeInTheDocument()
    expect(
      screen.getByText(/^Lege fest, welche Fachmodule in dieser Organisation und darunter/),
    ).toBeInTheDocument()
    expect(screen.getByRole('switch', { name: 'Bewässerung' })).toBeChecked()
    // A key the backend registers but this build ships no manifest for.
    expect(screen.getByRole('switch', { name: 'waste' })).not.toBeChecked()
  })

  it('says where each switch comes from, like the settings rows', () => {
    render(
      <OrganizationModulesSection
        draft={draftOf()}
        canUpdate
        organizationName={organizationName}
      />,
    )

    expect(screen.getByText('Geerbt von Stadt Flensburg')).toBeInTheDocument()
    expect(screen.getByText('Eigener Wert')).toBeInTheDocument()
  })

  it('hands a flipped switch to the draft as an own value', async () => {
    render(
      <OrganizationModulesSection
        draft={draftOf()}
        canUpdate
        organizationName={organizationName}
      />,
    )

    await userEvent.click(screen.getByRole('switch', { name: 'Bewässerung' }))
    expect(setOwn).toHaveBeenCalledWith('probe', false)
  })

  it('gives an own switch back to inheritance', async () => {
    render(
      <OrganizationModulesSection
        draft={draftOf()}
        canUpdate
        organizationName={organizationName}
      />,
    )

    // Only the row with an own value offers it.
    expect(
      screen.queryByRole('button', { name: 'Bewässerung wieder erben' }),
    ).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'waste wieder erben' }))
    expect(setOwn).toHaveBeenCalledWith('waste', null)
  })

  it('reads as a plain overview without setting:update', () => {
    render(
      <OrganizationModulesSection
        draft={draftOf()}
        canUpdate={false}
        organizationName={organizationName}
      />,
    )

    expect(screen.getByRole('switch', { name: 'Bewässerung' })).toBeDisabled()
    expect(screen.getByRole('switch', { name: 'waste' })).toBeDisabled()
    expect(screen.queryByRole('button', { name: /wieder erben/ })).not.toBeInTheDocument()
    expect(screen.getByText(/ansehen, aber nicht ändern/)).toBeInTheDocument()
  })

  it('names the locking organization and leaves every switch alone', () => {
    const rows = ROWS.map((row) => ({ ...row, locked: true }))
    render(
      <OrganizationModulesSection
        draft={draftOf({ rows, enforcedBy: { id: 'root' } })}
        canUpdate
        organizationName={organizationName}
      />,
    )

    expect(screen.getByRole('alert')).toHaveTextContent(/Stadt Flensburg/)
    expect(screen.getByRole('switch', { name: 'Bewässerung' })).toBeDisabled()
    expect(screen.queryByRole('button', { name: /wieder erben/ })).not.toBeInTheDocument()
  })

  it('names the organization that set a switch and keeps it fixed for the sub-unit', () => {
    const rows = [ROWS[0], { ...ROWS[1], setBy: 'root', locked: true }]
    render(
      <OrganizationModulesSection
        draft={draftOf({ rows })}
        canUpdate
        organizationName={organizationName}
      />,
    )

    expect(screen.getByText('Festgelegt von Stadt Flensburg')).toBeInTheDocument()
    expect(screen.queryByText('Eigener Wert')).not.toBeInTheDocument()
    expect(screen.getByRole('switch', { name: 'waste' })).toBeDisabled()
    expect(screen.getByRole('switch', { name: 'Bewässerung' })).toBeEnabled()
    expect(screen.queryByRole('button', { name: /waste wieder erben/ })).not.toBeInTheDocument()
  })

  it('names the setting organization to the parent, who may still change it', () => {
    const rows = [ROWS[0], { ...ROWS[1], setBy: 'root' }]
    render(
      <OrganizationModulesSection
        draft={draftOf({ rows })}
        canUpdate
        organizationName={organizationName}
      />,
    )

    expect(screen.getByText('Festgelegt von Stadt Flensburg')).toBeInTheDocument()
    expect(screen.getByRole('switch', { name: 'waste' })).toBeEnabled()
  })
})
