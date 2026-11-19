import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import {
  irrigationValidateJustWateredTtlHours,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import type { IrrigationSettingsResponse } from '@/api/backendApi'
import IrrigationSettingsSection from './IrrigationSettingsSection'
import type { IrrigationDraft, IrrigationSettingsDraft } from './useIrrigationSettingsDraft'

const IRRIGATION: IrrigationSettingsResponse = {
  waterDemand: { value: 120, inheritedValue: 80 },
  justWateredTtlSecs: { value: 43_200, inheritedValue: 86_400 },
}

const DRAFT: IrrigationDraft = { waterDemand: '120', justWateredTtlHours: '12' }

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
  setText,
  ...overrides,
})

const props = (overrides: Partial<Parameters<typeof IrrigationSettingsSection>[0]> = {}) => ({
  draft: draftOf(),
  locked: false,
  ...overrides,
})

describe('IrrigationSettingsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('shows both fields with their value and unit', () => {
    render(<IrrigationSettingsSection {...props()} />)

    expect(screen.getByRole('spinbutton', { name: /Wasserbedarf/i })).toHaveValue(120)
    expect(screen.getByRole('spinbutton', { name: /Frisch gegossen/i })).toHaveValue(12)
    expect(screen.getByText('Liter')).toBeInTheDocument()
    expect(screen.getByText('Stunden')).toBeInTheDocument()
  })

  it('names the usual value of each field', () => {
    render(<IrrigationSettingsSection {...props()} />)

    expect(screen.getByText('Üblich sind 80 Liter.')).toBeInTheDocument()
    expect(screen.getByText('Üblich sind 24 Stunden.')).toBeInTheDocument()
  })

  it('hands every keystroke to the draft instead of coercing it', async () => {
    render(<IrrigationSettingsSection {...props()} />)

    await userEvent.type(screen.getByRole('spinbutton', { name: /Frisch gegossen/i }), '8')
    expect(setText).toHaveBeenCalledWith('justWateredTtlHours', '128')
  })

  it('disables both inputs without setting:update', () => {
    render(<IrrigationSettingsSection {...props({ locked: true })} />)

    for (const input of screen.getAllByRole('spinbutton')) expect(input).toBeDisabled()
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

    expect(
      screen.getByRole('spinbutton', { name: /Frisch gegossen/i }),
    ).toHaveAccessibleDescription(/^Nachwirkzeit muss zwischen 1 und 336 liegen\./)
  })
})
