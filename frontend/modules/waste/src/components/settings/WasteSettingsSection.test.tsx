import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import { wasteValidateFullThreshold, type ValidationIssue } from '@green-ecolution/domain-wasm'
import type { WasteSettingsResponse } from '@green-ecolution/backend-client'
import WasteSettingsSection from './WasteSettingsSection'
import type { WasteSettingsDraft } from './useWasteSettingsDraft'

const server: WasteSettingsResponse = { fullThreshold: { value: 70, inheritedValue: 80 } }

const setText = vi.fn()

const draftOf = (overrides: Partial<WasteSettingsDraft> = {}): WasteSettingsDraft => ({
  loading: false,
  loadError: null,
  dirty: false,
  valid: true,
  reset: vi.fn(),
  save: vi.fn(),
  server,
  draft: { text: '70' },
  issue: null,
  setText,
  ...overrides,
})

const props = (overrides: Partial<Parameters<typeof WasteSettingsSection>[0]> = {}) => ({
  draft: draftOf(),
  locked: false,
  ...overrides,
})

describe('WasteSettingsSection', () => {
  beforeEach(() => vi.clearAllMocks())

  it('shows the value with its unit and the usual value', () => {
    render(<WasteSettingsSection {...props()} />)

    expect(screen.getByRole('spinbutton', { name: /voll/i })).toHaveValue(70)
    expect(screen.getByText('Prozent')).toBeInTheDocument()
    expect(screen.getByText('Üblich sind 80 Prozent.')).toBeInTheDocument()
  })

  it('hands every keystroke to the draft', async () => {
    render(<WasteSettingsSection {...props()} />)

    await userEvent.type(screen.getByRole('spinbutton', { name: /voll/i }), '5')
    expect(setText).toHaveBeenCalledWith('705')
  })

  it('disables the input without setting:update', () => {
    render(<WasteSettingsSection {...props({ locked: true })} />)

    expect(screen.getByRole('spinbutton', { name: /voll/i })).toBeDisabled()
  })

  it('reports a range error at the field itself', () => {
    const issue = wasteValidateFullThreshold(0) as ValidationIssue
    render(<WasteSettingsSection {...props({ draft: draftOf({ draft: { text: '0' }, issue }) })} />)

    expect(screen.getByRole('spinbutton', { name: /voll/i })).toHaveAccessibleDescription(
      /^Voll-Schwelle muss zwischen 1 und 100 Prozent liegen\./,
    )
  })
})
