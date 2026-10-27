import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import SensorReviewStep from './SensorReviewStep'

const target = {
  target: { module: 'probe', kind: 'probe', id: 'p-1' },
  label: 'Messpunkt 0815',
  detail: 'Nordufer',
}

const fix = {
  latitude: 54.79123,
  longitude: 9.43456,
  accuracy: 6.4,
  timestamp: 1_710_000_000_000,
}

describe('SensorReviewStep', () => {
  it('renders sensor id, the picked target and coordinates', () => {
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        target={target}
        position={fix}
        status="idle"
        errorMessage={null}
        onActivate={vi.fn()}
      />,
    )
    expect(screen.getByText('EUI-001')).toBeInTheDocument()
    expect(screen.getByText('Messpunkt 0815')).toBeInTheDocument()
    expect(screen.getByText('Nordufer')).toBeInTheDocument()
    expect(screen.getByText(/54\.79/)).toBeInTheDocument()
    expect(screen.getByText(/9\.43/)).toBeInTheDocument()
  })

  it('capitalises a target label worded for mid-sentence use', () => {
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        target={{ ...target, label: 'tree 0815' }}
        position={null}
        status="idle"
        errorMessage={null}
        onActivate={vi.fn()}
      />,
    )
    expect(screen.getByText('Tree 0815')).toBeInTheDocument()
  })

  it('fires onActivate when the primary button is clicked', async () => {
    const user = userEvent.setup()
    const onActivate = vi.fn()
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        target={target}
        position={fix}
        status="idle"
        errorMessage={null}
        onActivate={onActivate}
      />,
    )
    await user.click(screen.getByRole('button', { name: /sensor aktivieren/i }))
    expect(onActivate).toHaveBeenCalledTimes(1)
  })

  it('disables the button while submission is pending', () => {
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        target={target}
        position={fix}
        status="pending"
        errorMessage={null}
        onActivate={vi.fn()}
      />,
    )
    expect(screen.getByRole('button', { name: /aktiviere|aktivieren/i })).toBeDisabled()
  })

  it('shows the error message when submission failed', () => {
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        target={target}
        position={fix}
        status="error"
        errorMessage="Aktivierung fehlgeschlagen"
        onActivate={vi.fn()}
      />,
    )
    expect(screen.getByText(/aktivierung fehlgeschlagen/i)).toBeInTheDocument()
  })
})
