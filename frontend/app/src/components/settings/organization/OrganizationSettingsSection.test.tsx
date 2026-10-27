import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import type { SettingsDraft } from './useOrganizationSettingsDraft'

// MapLibre needs a WebGL context, which jsdom does not provide.
vi.mock('@/components/map-gl/MapPreview', () => ({
  default: () => <div data-testid="map-preview" />,
}))

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

const onMapViewOwnChange = vi.fn()
const onMapViewChange = vi.fn()
const onMapViewRestrictedChange = vi.fn()
const onMapViewZoomChange = vi.fn()
const onDescendantsMayOverrideChange = vi.fn()

const props = (overrides: Partial<Record<string, unknown>> = {}) => ({
  settings: SETTINGS,
  draft: DRAFT,
  errors: { mapView: null },
  canUpdate: true,
  enforcedByName: null,
  sourceNames: { mapView: null },
  onMapViewOwnChange,
  onMapViewChange,
  onMapViewRestrictedChange,
  onMapViewZoomChange,
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

  it('places the module sections inside the card, ahead of the map viewport', () => {
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
      section.compareDocumentPosition(screen.getByText('Kartenausschnitt')) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
  })

  it('toggles the lock for sub-units', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.click(
      screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
    )
    expect(onDescendantsMayOverrideChange).toHaveBeenCalledWith(false)
  })

  it('shows the map viewport with its centre and offers to take it over', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    expect(screen.getByText('Kartenausschnitt')).toBeInTheDocument()
    expect(screen.getByText(/Mittelpunkt 54\.7923, 9\.4358/)).toBeInTheDocument()

    await userEvent.click(
      screen.getByRole('button', { name: /Eigenen Wert für Kartenausschnitt setzen/i }),
    )
    expect(onMapViewOwnChange).toHaveBeenCalledWith(true)
  })

  it('reports an invalid viewport at the row itself', () => {
    render(
      <OrganizationSettingsSection
        {...props({
          errors: {
            mapView: 'Der Mittelpunkt muss innerhalb des gewählten Ausschnitts liegen.',
          },
        })}
      />,
    )

    expect(
      screen.getByRole('alert', {
        name: '',
      }),
    ).toHaveTextContent('Der Mittelpunkt muss innerhalb des gewählten Ausschnitts liegen.')
  })

  // The limit belongs to the value: while that is inherited there is nothing
  // here to lift, and taking it over comes first.
  it('lets the limit be lifted only once the organization owns the viewport', async () => {
    render(<OrganizationSettingsSection {...props()} />)
    expect(screen.getByRole('switch', { name: /Keine Einschränkung/ })).toBeDisabled()

    const draft: SettingsDraft = { ...DRAFT, mapView: { own: true, value: MAP_VIEW } }
    render(<OrganizationSettingsSection {...props({ draft })} />)

    const switches = screen.getAllByRole('switch', { name: /Keine Einschränkung/ })
    const own = switches[switches.length - 1]
    expect(own).not.toBeChecked()

    await userEvent.click(own)
    expect(onMapViewRestrictedChange).toHaveBeenCalledWith(false)
  })

  it('shows the limit as lifted when the viewport carries no box', () => {
    const settings = {
      ...SETTINGS,
      mapView: { value: { center: MAP_VIEW.center, bbox: null }, origin: 'own', lastChange: null },
    } as unknown as OrganizationSettingsResponse
    const draft: SettingsDraft = {
      ...DRAFT,
      mapView: { own: true, value: { center: MAP_VIEW.center, bbox: null } },
    }
    render(<OrganizationSettingsSection {...props({ settings, draft })} />)

    expect(screen.getByRole('switch', { name: /Keine Einschränkung/ })).toBeChecked()
    // Without a limit the extent is meaningless, so the action speaks of the
    // centre alone.
    expect(screen.getByRole('button', { name: 'Mittelpunkt übernehmen' })).toBeInTheDocument()
  })

  it('leaves the viewport a fact to read while an ancestor holds the lock', () => {
    render(<OrganizationSettingsSection {...enforced()} />)

    expect(
      screen.queryByRole('button', { name: /Eigenen Wert für Kartenausschnitt setzen/i }),
    ).not.toBeInTheDocument()
  })
})
