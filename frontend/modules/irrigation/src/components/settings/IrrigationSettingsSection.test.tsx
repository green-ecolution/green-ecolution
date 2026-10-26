import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import {
  irrigationValidateJustWateredTtlHours,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import type { IrrigationSettingsResponse } from '@/api/backendApi'
import IrrigationSettingsSection from './IrrigationSettingsSection'
import type { IrrigationDraft, IrrigationSettingsDraft } from './useIrrigationSettingsDraft'

const IRRIGATION = {
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
  enforcedBy: null,
} as unknown as IrrigationSettingsResponse

const DRAFT: IrrigationDraft = {
  waterDemand: { own: false, text: '80' },
  justWateredTtlHours: { own: true, text: '24' },
}

const setOwn = vi.fn()
const setText = vi.fn()

const draftOf = (overrides: Partial<IrrigationSettingsDraft> = {}): IrrigationSettingsDraft => ({
  loading: false,
  loadError: null,
  dirty: false,
  valid: true,
  reset: vi.fn(),
  save: vi.fn(),
  server: IRRIGATION,
  draft: DRAFT,
  issues: { waterDemand: null, justWateredTtlHours: null },
  setOwn,
  setText,
  ...overrides,
})

const names: Record<string, string> = { root: 'Stadt Flensburg' }
const organizationName = (id: string | null | undefined) => (id ? (names[id] ?? null) : null)

const props = (overrides: Partial<Parameters<typeof IrrigationSettingsSection>[0]> = {}) => ({
  draft: draftOf(),
  locked: false,
  enforced: false,
  organizationName,
  ...overrides,
})

// An ancestor froze the subtree: the effective values come from there and this
// organization's own value lies dormant.
const enforced = () => {
  const server = {
    ...IRRIGATION,
    justWateredTtlSecs: {
      value: 86400,
      origin: 'inherited',
      source: { id: 'root' },
      ownValue: 172800,
      lastChange: null,
    },
  } as unknown as IrrigationSettingsResponse
  const draft: IrrigationDraft = { ...DRAFT, justWateredTtlHours: { own: false, text: '24' } }
  return props({ draft: draftOf({ server, draft }), locked: true, enforced: true })
}

describe('IrrigationSettingsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('names the organization an inherited value comes from', () => {
    render(<IrrigationSettingsSection {...props()} />)

    expect(screen.getByText(/Geerbt von Stadt Flensburg/i)).toBeInTheDocument()
    expect(screen.getByText('Eigener Wert')).toBeInTheDocument()
  })

  // The badge names where the stored value comes from, which an unsaved toggle
  // has not changed yet; colour and wording must not tell different stories.
  it('keeps the origin badge on the stored origin while a takeover is unsaved', () => {
    const draft: IrrigationDraft = { ...DRAFT, waterDemand: { own: true, text: '90' } }
    render(<IrrigationSettingsSection {...props({ draft: draftOf({ draft }) })} />)

    const badge = screen.getByText(/Geerbt von Stadt Flensburg/i)
    expect(badge.className).not.toMatch(/green/)
  })

  // An inherited value is a statement of where it comes from, not a greyed-out
  // input: there is nothing to type into until the organization takes it over.
  it('states an inherited value instead of offering a field for it', async () => {
    render(<IrrigationSettingsSection {...props()} />)

    expect(screen.queryByRole('spinbutton', { name: /Wasserbedarf/i })).not.toBeInTheDocument()
    expect(screen.getByText('80 Liter')).toBeInTheDocument()
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toBeEnabled()

    // The accessible name carries the field, so the two rows stay tellable apart.
    await userEvent.click(
      screen.getByRole('button', { name: 'Eigenen Wert für Wasserbedarf je Baum setzen' }),
    )
    expect(setOwn).toHaveBeenCalledWith('waterDemand', true)
  })

  it('says what each value actually controls', () => {
    render(<IrrigationSettingsSection {...props()} />)

    expect(screen.getByText(/Bedarfsrechnung jeder Bewässerungsgruppe/)).toBeInTheDocument()
    expect(screen.getByText(/als soeben bewässert/)).toBeInTheDocument()
  })

  it('gives an own value back to inheritance', async () => {
    render(<IrrigationSettingsSection {...props()} />)

    await userEvent.click(
      screen.getByRole('button', { name: 'Nachwirkzeit frisch gegossen wieder erben' }),
    )
    expect(setOwn).toHaveBeenCalledWith('justWateredTtlHours', false)
  })

  it('hands every keystroke to the draft instead of coercing it', async () => {
    render(<IrrigationSettingsSection {...props()} />)

    await userEvent.type(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i }), '8')
    expect(setText).toHaveBeenCalledWith('justWateredTtlHours', '248')
  })

  it('reads as a plain overview without setting:update', () => {
    render(<IrrigationSettingsSection {...props({ locked: true })} />)

    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument()
    expect(screen.getByText('80 Liter')).toBeInTheDocument()
    expect(screen.getByText('24 Stunden')).toBeInTheDocument()
    // No dead controls: the actions are gone.
    expect(screen.queryByRole('button', { name: /wieder erben/i })).not.toBeInTheDocument()
  })

  it('keeps the dormant own value visible while an ancestor holds the lock', () => {
    render(<IrrigationSettingsSection {...enforced()} />)

    expect(screen.queryByRole('spinbutton', { name: /Nachwirkzeit/i })).not.toBeInTheDocument()
    expect(screen.getByText('24 Stunden')).toBeInTheDocument()
    expect(screen.getByText(/48 Stunden.*ruht/i)).toBeInTheDocument()
  })

  it('shows when and by whom a value was last changed', () => {
    render(<IrrigationSettingsSection {...props()} />)

    expect(screen.getByText(/12\.03\.2026.*ge\.admin/)).toBeInTheDocument()
  })

  it('reports a range error at the field itself', () => {
    const issue = irrigationValidateJustWateredTtlHours(9999) as ValidationIssue
    render(
      <IrrigationSettingsSection
        {...props({
          draft: draftOf({ issues: { waterDemand: null, justWateredTtlHours: issue } }),
        })}
      />,
    )

    // The description also carries what the value does and its unit; the error
    // comes first, so it is the first thing announced.
    expect(screen.getByRole('spinbutton', { name: /Nachwirkzeit/i })).toHaveAccessibleDescription(
      /^Nachwirkzeit muss zwischen 1 und 336 liegen\./,
    )
  })
})
