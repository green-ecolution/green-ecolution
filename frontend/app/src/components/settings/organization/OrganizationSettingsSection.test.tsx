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
  waterDemand: {
    value: 80,
    origin: 'inherited',
    source: { id: 'root' },
    lastChange: null,
  },
  justWateredTtlSecs: {
    value: 86400,
    origin: 'own',
    ownValue: 86400,
    lastChange: {
      changedAt: new Date('2026-03-12T09:00:00Z'),
      changedBy: 'u-admin',
      changedByName: 'ge.admin',
    },
  },
  mapView: {
    value: MAP_VIEW,
    origin: 'default',
    lastChange: null,
  },
  descendantsMayOverride: true,
  enforcedBy: null,
} as unknown as OrganizationSettingsResponse

const DRAFT: SettingsDraft = {
  waterDemand: { own: false, text: '80' },
  justWateredTtlHours: { own: true, text: '24' },
  mapView: { own: false, value: MAP_VIEW },
  descendantsMayOverride: true,
}

const onOwnChange = vi.fn()
const onTextChange = vi.fn()
const onMapViewOwnChange = vi.fn()
const onMapViewChange = vi.fn()
const onMapViewRestrictedChange = vi.fn()
const onMapViewZoomChange = vi.fn()
const onDescendantsMayOverrideChange = vi.fn()

const props = (overrides: Partial<Record<string, unknown>> = {}) => ({
  settings: SETTINGS,
  draft: DRAFT,
  errors: { waterDemand: null, justWateredTtlHours: null, mapView: null },
  canUpdate: true,
  enforcedByName: null,
  sourceNames: { waterDemand: 'Stadt Flensburg', justWateredTtlHours: null, mapView: null },
  onOwnChange,
  onTextChange,
  onMapViewOwnChange,
  onMapViewChange,
  onMapViewRestrictedChange,
  onMapViewZoomChange,
  onDescendantsMayOverrideChange,
  ...overrides,
})

// An ancestor froze the subtree: the effective values come from there and this
// organization's own value lies dormant.
const enforced = () => {
  const settings = {
    ...SETTINGS,
    enforcedBy: { id: 'root' },
    justWateredTtlSecs: {
      value: 86400,
      origin: 'inherited',
      source: { id: 'root' },
      ownValue: 172800,
      lastChange: null,
    },
  } as unknown as OrganizationSettingsResponse
  const draft: SettingsDraft = {
    ...DRAFT,
    justWateredTtlHours: { own: false, text: '24' },
  }
  return props({ settings, draft, enforcedByName: 'Stadt Flensburg' })
}

describe('OrganizationSettingsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('names the organization an inherited value comes from', () => {
    render(<OrganizationSettingsSection {...props()} />)

    expect(screen.getByText(/Geerbt von Stadt Flensburg/i)).toBeInTheDocument()
    expect(screen.getByText('Eigener Wert')).toBeInTheDocument()
  })

  // The badge names where the stored value comes from, which an unsaved toggle
  // has not changed yet; colour and wording must not tell different stories.
  it('keeps the origin badge on the stored origin while a takeover is unsaved', () => {
    const draft: SettingsDraft = { ...DRAFT, waterDemand: { own: true, text: '90' } }
    render(<OrganizationSettingsSection {...props({ draft })} />)

    const badge = screen.getByText(/Geerbt von Stadt Flensburg/i)
    expect(badge.className).not.toMatch(/green/)
  })

  // An inherited value is a statement of where it comes from, not a greyed-out
  // input: there is nothing to type into until the organization takes it over.
  it('states an inherited value instead of offering a field for it', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    expect(screen.queryByRole('spinbutton', { name: /Wasserbedarf/i })).not.toBeInTheDocument()
    expect(screen.getByText('80 Liter')).toBeInTheDocument()
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toBeEnabled()

    // The accessible name carries the field, so the two rows stay tellable apart.
    await userEvent.click(
      screen.getByRole('button', { name: 'Eigenen Wert für Wasserbedarf je Baum setzen' }),
    )
    expect(onOwnChange).toHaveBeenCalledWith('waterDemand', true)
  })

  it('says what each value actually controls', () => {
    render(<OrganizationSettingsSection {...props()} />)

    expect(screen.getByText(/Bedarfsrechnung jeder Bewässerungsgruppe/)).toBeInTheDocument()
    expect(screen.getByText(/als soeben bewässert/)).toBeInTheDocument()
  })

  it('gives an own value back to inheritance', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.click(
      screen.getByRole('button', { name: 'Nachwirkzeit frisch gegossen wieder erben' }),
    )
    expect(onOwnChange).toHaveBeenCalledWith('justWateredTtlHours', false)
  })

  it('hands every keystroke to the draft instead of coercing it', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.type(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i }), '8')
    expect(onTextChange).toHaveBeenCalledWith('justWateredTtlHours', '248')
  })

  it('reads as a plain overview without setting:update', () => {
    render(<OrganizationSettingsSection {...props({ canUpdate: false })} />)

    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument()
    expect(screen.getByText('80 Liter')).toBeInTheDocument()
    expect(screen.getByText('24 Stunden')).toBeInTheDocument()
    expect(
      screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
    ).toBeDisabled()
    // No dead controls: the actions are gone and a line says why.
    expect(screen.queryByRole('button', { name: /wieder erben/i })).not.toBeInTheDocument()
    expect(screen.getByText(/ansehen, aber nicht ändern/)).toBeInTheDocument()
  })

  it('shows the locking organization and keeps the dormant own value visible', () => {
    render(<OrganizationSettingsSection {...enforced()} />)

    expect(screen.getByRole('alert')).toHaveTextContent(/Stadt Flensburg/)
    expect(screen.queryByRole('spinbutton', { name: /Nachwirkzeit/i })).not.toBeInTheDocument()
    expect(screen.getByText('24 Stunden')).toBeInTheDocument()
    expect(screen.getByText(/48 Stunden.*ruht/i)).toBeInTheDocument()
    // The lock covers the switch too — the backend refuses it as well.
    expect(
      screen.getByRole('switch', { name: /Untereinheiten dürfen eigene Werte setzen/ }),
    ).toBeDisabled()
  })

  it('shows when and by whom a value was last changed', () => {
    render(<OrganizationSettingsSection {...props()} />)

    expect(screen.getByText(/12\.03\.2026.*ge\.admin/)).toBeInTheDocument()
  })

  it('reports a range error at the field itself', () => {
    render(
      <OrganizationSettingsSection
        {...props({
          errors: {
            waterDemand: null,
            justWateredTtlHours: 'Nachwirkzeit muss zwischen 1 und 336 liegen.',
          },
        })}
      />,
    )

    // The description also carries what the value does and its unit; the error
    // comes first, so it is the first thing announced.
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toHaveAccessibleDescription(
      /^Nachwirkzeit muss zwischen 1 und 336 liegen\./,
    )
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
            waterDemand: null,
            justWateredTtlHours: null,
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
