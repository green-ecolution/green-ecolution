import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import type { SettingsDraft } from './useOrganizationSettingsDraft'

const { default: OrganizationSettingsSection } = await import('./OrganizationSettingsSection')

const MAP_VIEW = {
  center: [54.7923, 9.4358],
  bbox: [54.7148, 9.2858, 54.8601, 9.5838],
  minZoom: 13,
  maxZoom: 18,
}

const SETTINGS = {
  mapView: {
    value: MAP_VIEW,
    origin: 'default',
    lastChange: null,
  },
  descendantsMayOverride: true,
  enforcedBy: null,
} as unknown as OrganizationSettingsResponse

const DRAFT: SettingsDraft = {
  mapView: { own: false, value: MAP_VIEW },
  descendantsMayOverride: true,
}

const onDescendantsMayOverrideChange = vi.fn()

const props = (overrides: Partial<Record<string, unknown>> = {}) => ({
  settings: SETTINGS,
  draft: DRAFT,
  canUpdate: true,
  enforcedByName: null,
  onDescendantsMayOverrideChange,
  ...overrides,
})

// An ancestor froze the subtree: the effective values come from there.
const enforced = () => {
  const settings = {
    ...SETTINGS,
    enforcedBy: { id: 'root' },
  } as unknown as OrganizationSettingsResponse
  return props({ settings, enforcedByName: 'Stadt Flensburg' })
}

describe('OrganizationSettingsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('reads as a plain overview without setting:update', () => {
    render(<OrganizationSettingsSection {...props({ canUpdate: false })} />)

    expect(
      screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
    ).toBeDisabled()
    // No dead controls: the actions are gone and a line says why.
    expect(screen.queryByRole('button', { name: /wieder erben/i })).not.toBeInTheDocument()
    expect(screen.getByText(/ansehen, aber nicht ändern/)).toBeInTheDocument()
  })

  it('shows the locking organization', () => {
    render(<OrganizationSettingsSection {...enforced()} />)

    expect(screen.getByRole('alert')).toHaveTextContent(/Stadt Flensburg/)
    // The lock covers the switch too — the backend refuses it as well.
    expect(
      screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
    ).toBeDisabled()
  })

  it('places the module sections inside the card, ahead of the lock switch', () => {
    render(
      <OrganizationSettingsSection
        {...props({ moduleSections: <p data-testid="module-section">Modul</p> })}
      />,
    )

    const section = screen.getByTestId('module-section')
    expect(section.closest('section')).toBe(
      screen.getByRole('heading', { name: 'Fachliche Vorgaben' }).closest('section'),
    )
    expect(
      section.compareDocumentPosition(
        screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
      ) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('toggles the lock for sub-units', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.click(
      screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
    )
    expect(onDescendantsMayOverrideChange).toHaveBeenCalledWith(false)
  })
})
