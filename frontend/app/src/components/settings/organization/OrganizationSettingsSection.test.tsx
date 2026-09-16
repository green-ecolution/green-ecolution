import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import OrganizationSettingsSection from './OrganizationSettingsSection'
import type { SettingsDraft } from './useOrganizationSettingsDraft'

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
  descendantsMayOverride: true,
  enforcedBy: null,
} as unknown as OrganizationSettingsResponse

const DRAFT: SettingsDraft = {
  waterDemand: { own: false, text: '80' },
  justWateredTtlHours: { own: true, text: '24' },
  descendantsMayOverride: true,
}

const onOwnChange = vi.fn()
const onTextChange = vi.fn()
const onDescendantsMayOverrideChange = vi.fn()

const props = (overrides: Partial<Record<string, unknown>> = {}) => ({
  settings: SETTINGS,
  draft: DRAFT,
  errors: { waterDemand: null, justWateredTtlHours: null },
  canUpdate: true,
  enforcedByName: null,
  sourceNames: { waterDemand: 'Stadt Flensburg', justWateredTtlHours: null },
  onOwnChange,
  onTextChange,
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

  it('keeps an inherited field out of reach until it is taken over', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    expect(screen.getByRole('spinbutton', { name: /Wasserbedarf/i })).toBeDisabled()
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toBeEnabled()

    // The accessible name carries the field, so the two rows stay tellable apart.
    await userEvent.click(
      screen.getByRole('button', { name: 'Eigenen Wert für Wasserbedarf je Baum (Liter) setzen' }),
    )
    expect(onOwnChange).toHaveBeenCalledWith('waterDemand', true)
  })

  it('gives an own value back to inheritance', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.click(
      screen.getByRole('button', { name: 'Nachwirkzeit frisch gegossen (Stunden) wieder erben' }),
    )
    expect(onOwnChange).toHaveBeenCalledWith('justWateredTtlHours', false)
  })

  it('hands every keystroke to the draft instead of coercing it', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.type(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i }), '8')
    expect(onTextChange).toHaveBeenCalledWith('justWateredTtlHours', '248')
  })

  it('disables the inputs without setting:update', () => {
    render(<OrganizationSettingsSection {...props({ canUpdate: false })} />)

    expect(screen.getByRole('spinbutton', { name: /Wasserbedarf/i })).toBeDisabled()
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toBeDisabled()
    expect(screen.getByRole('switch')).toBeDisabled()
    expect(screen.getByRole('button', { name: /wieder erben/i })).toBeDisabled()
  })

  it('shows the locking organization and keeps the dormant own value visible', () => {
    render(<OrganizationSettingsSection {...enforced()} />)

    expect(screen.getByRole('alert')).toHaveTextContent(/Stadt Flensburg/)
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toHaveValue(24)
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toBeDisabled()
    expect(screen.getByText(/48 Stunden.*ruht/i)).toBeInTheDocument()
    // The lock covers the switch too — the backend refuses it as well.
    expect(screen.getByRole('switch')).toBeDisabled()
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

    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toHaveAccessibleDescription(
      'Nachwirkzeit muss zwischen 1 und 336 liegen.',
    )
  })

  it('toggles the lock for sub-units', async () => {
    render(<OrganizationSettingsSection {...props()} />)

    await userEvent.click(screen.getByRole('switch'))
    expect(onDescendantsMayOverrideChange).toHaveBeenCalledWith(false)
  })
})
