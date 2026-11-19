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
        toggle={{ enabled: true, disabled: false, note: null, onChange }}
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
        toggle={{ enabled: false, disabled: true, note: 'Legt TBZ fest.', onChange: vi.fn() }}
      />,
    )
    expect(screen.getByText('Ausgeschaltet')).toBeInTheDocument()
    expect(screen.getByRole('switch', { name: 'Abfall' })).toBeDisabled()
    expect(screen.getByText('Legt TBZ fest.')).toBeInTheDocument()
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
