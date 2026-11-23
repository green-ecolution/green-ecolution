import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import ModuleAreaCard from './ModuleAreaCard'

const Icon = () => null

describe('ModuleAreaCard', () => {
  it('names the state next to the switch and reports a flip', async () => {
    const onChange = vi.fn()
    render(
      <ModuleAreaCard
        icon={Icon}
        title="Bewässerung"
        description="Bäume und Gießeinsätze"
        toggle={{ enabled: true, note: null, control: { disabled: false, onChange } }}
      />,
    )
    expect(screen.getByText('Eingeschaltet')).toBeInTheDocument()
    await userEvent.click(screen.getByRole('switch', { name: 'Bewässerung' }))
    expect(onChange).toHaveBeenCalledWith(false)
  })

  it('says "Ausgeschaltet" and shows a note for a fixed switch', () => {
    render(
      <ModuleAreaCard
        icon={Icon}
        title="Abfall"
        description={null}
        toggle={{
          enabled: false,
          note: 'Legt TBZ fest.',
          control: { disabled: true, onChange: vi.fn() },
        }}
      />,
    )
    expect(screen.getByText('Ausgeschaltet')).toBeInTheDocument()
    expect(screen.getByRole('switch', { name: 'Abfall' })).toBeDisabled()
    expect(screen.getByText('Legt TBZ fest.')).toBeInTheDocument()
    expect(screen.getByRole('switch', { name: 'Abfall' })).toHaveAccessibleDescription(
      'Legt TBZ fest.',
    )
  })

  it('shows the state and the note without a switch when there is no control', () => {
    render(
      <ModuleAreaCard
        icon={Icon}
        title="Abfall"
        description={null}
        toggle={{ enabled: true, note: 'Schaltet die Mutter.' }}
      />,
    )
    expect(screen.queryByRole('switch')).toBeNull()
    expect(screen.getByText('Eingeschaltet')).toBeInTheDocument()
    expect(screen.getByText('Schaltet die Mutter.')).toBeInTheDocument()
  })

  it('has no switch without a toggle and renders its fields', () => {
    render(
      <ModuleAreaCard icon={Icon} title="Sensoren" description={null}>
        <p>Feld</p>
      </ModuleAreaCard>,
    )
    expect(screen.queryByRole('switch')).toBeNull()
    expect(screen.getByText('Feld')).toBeInTheDocument()
  })
})
