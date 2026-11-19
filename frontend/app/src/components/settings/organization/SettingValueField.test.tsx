import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import SettingValueField from './SettingValueField'

const props = {
  label: 'Wasserbedarf je Baum',
  description: 'So viel Wasser rechnet die Tourenplanung für jeden Baum ein.',
  unit: 'Liter',
  usual: '80 Liter',
  usualCount: 80,
  text: '120',
  error: null,
  disabled: false,
  min: 1,
  max: 500,
  step: 1,
  onText: vi.fn(),
}

describe('SettingValueField', () => {
  it('labels the input and states the usual value', () => {
    render(<SettingValueField {...props} />)
    expect(screen.getByLabelText('Wasserbedarf je Baum')).toHaveValue(120)
    expect(screen.getByText('Üblich sind 80 Liter.')).toBeInTheDocument()
  })

  it('uses the singular sentence for a single unit', () => {
    render(<SettingValueField {...props} usual="1 Stunde" usualCount={1} />)
    expect(screen.getByText('Üblich ist 1 Stunde.')).toBeInTheDocument()
  })

  it('reports what is typed', async () => {
    const onText = vi.fn()
    render(<SettingValueField {...props} text="" onText={onText} />)
    await userEvent.type(screen.getByLabelText('Wasserbedarf je Baum'), '9')
    expect(onText).toHaveBeenCalledWith('9')
  })

  it('ties an error to the input and announces it', () => {
    render(<SettingValueField {...props} error="Zu groß." />)
    const input = screen.getByLabelText('Wasserbedarf je Baum')
    expect(input).toHaveAttribute('aria-invalid', 'true')
    expect(screen.getByRole('alert')).toHaveTextContent('Zu groß.')
  })

  it('is not editable when disabled', () => {
    render(<SettingValueField {...props} disabled />)
    expect(screen.getByLabelText('Wasserbedarf je Baum')).toBeDisabled()
  })
})
