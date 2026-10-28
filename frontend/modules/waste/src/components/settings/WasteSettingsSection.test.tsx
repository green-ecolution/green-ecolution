import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import { wasteValidateFullThreshold, type ValidationIssue } from '@green-ecolution/domain-wasm'
import type { WasteSettingsResponse } from '@green-ecolution/backend-client'
import WasteSettingsSection from './WasteSettingsSection'
import type { WasteSettingsDraft } from './useWasteSettingsDraft'

const inheritedServer = {
  fullThreshold: { value: 80, origin: 'inherited', source: { id: 'root' }, lastChange: null },
  enforcedBy: null,
} as unknown as WasteSettingsResponse

const ownServer = {
  fullThreshold: {
    value: 70,
    origin: 'own',
    ownValue: 70,
    lastChange: {
      changedAt: new Date('2026-03-12T09:00:00Z'),
      changedBy: 'u-admin',
      changedByName: 'ge.admin',
    },
  },
  enforcedBy: null,
} as unknown as WasteSettingsResponse

const setOwn = vi.fn()
const setText = vi.fn()

const draftOf = (overrides: Partial<WasteSettingsDraft> = {}): WasteSettingsDraft => ({
  loading: false,
  loadError: null,
  dirty: false,
  valid: true,
  reset: vi.fn(),
  save: vi.fn(),
  server: inheritedServer,
  draft: { own: false, text: '80' },
  issue: null,
  setOwn,
  setText,
  ...overrides,
})

const names: Record<string, string> = { root: 'Stadt Flensburg' }
const organizationName = (id: string | null | undefined) => (id ? (names[id] ?? null) : null)

const props = (overrides: Partial<Parameters<typeof WasteSettingsSection>[0]> = {}) => ({
  draft: draftOf(),
  locked: false,
  enforced: false,
  organizationName,
  ...overrides,
})

describe('WasteSettingsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('names the organization an inherited value comes from', () => {
    render(<WasteSettingsSection {...props()} />)

    expect(screen.getByText(/Geerbt von Stadt Flensburg/i)).toBeInTheDocument()
    expect(screen.getByText('80 %')).toBeInTheDocument()
    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument()
  })

  it('offers to take the value over', async () => {
    render(<WasteSettingsSection {...props()} />)

    await userEvent.click(
      screen.getByRole('button', { name: /Eigenen Wert für Schwellwert für „voll“ setzen/ }),
    )
    expect(setOwn).toHaveBeenCalledWith(true)
  })

  it('hands every keystroke to the draft', async () => {
    render(
      <WasteSettingsSection
        {...props({
          draft: draftOf({ server: ownServer, draft: { own: true, text: '70' } }),
        })}
      />,
    )

    await userEvent.type(screen.getByRole('spinbutton', { name: /Schwellwert/i }), '5')
    expect(setText).toHaveBeenCalledWith('705')
  })

  it('shows when and by whom the value was last changed', () => {
    render(
      <WasteSettingsSection
        {...props({ draft: draftOf({ server: ownServer, draft: { own: true, text: '70' } }) })}
      />,
    )
    expect(screen.getByText(/12\.03\.2026.*ge\.admin/)).toBeInTheDocument()
  })

  it('reads as a plain overview and shows the origin when locked', () => {
    render(<WasteSettingsSection {...props({ locked: true, enforced: true })} />)

    expect(screen.queryByRole('spinbutton')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /setzen|erben/i })).not.toBeInTheDocument()
    expect(screen.getByText(/Geerbt von Stadt Flensburg/i)).toBeInTheDocument()
  })

  it('reports a range error at the field itself', () => {
    const issue = wasteValidateFullThreshold(0) as ValidationIssue
    render(
      <WasteSettingsSection
        {...props({
          draft: draftOf({ server: ownServer, draft: { own: true, text: '0' }, issue }),
        })}
      />,
    )

    expect(screen.getByRole('spinbutton', { name: /Schwellwert/i })).toHaveAccessibleDescription(
      /^Voll-Schwelle muss zwischen 1 und 100 Prozent liegen\./,
    )
  })
})
