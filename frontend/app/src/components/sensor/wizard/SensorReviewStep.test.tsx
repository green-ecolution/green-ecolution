import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import SensorReviewStep from './SensorReviewStep'

const fix = {
  latitude: 54.79123,
  longitude: 9.43456,
  accuracy: 6.4,
  timestamp: 1_710_000_000_000,
}

describe('SensorReviewStep', () => {
  it('renders sensor id, tree info and coordinates', () => {
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        treeNumber="0815"
        treeSpecies="Tilia cordata"
        position={fix}
        status="idle"
        errorMessage={null}
        onActivate={vi.fn()}
      />,
    )
    expect(screen.getByText('EUI-001')).toBeInTheDocument()
    expect(screen.getByText('0815')).toBeInTheDocument()
    expect(screen.getByText('Tilia cordata')).toBeInTheDocument()
    expect(screen.getByText(/54\.79/)).toBeInTheDocument()
    expect(screen.getByText(/9\.43/)).toBeInTheDocument()
  })

  it('fires onActivate when the primary button is clicked', async () => {
    const user = userEvent.setup()
    const onActivate = vi.fn()
    render(
      <SensorReviewStep
        sensorId="EUI-001"
        treeNumber="0815"
        treeSpecies="Tilia cordata"
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
        treeNumber="0815"
        treeSpecies="Tilia cordata"
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
        treeNumber="0815"
        treeSpecies="Tilia cordata"
        position={fix}
        status="error"
        errorMessage="Aktivierung fehlgeschlagen"
        onActivate={vi.fn()}
      />,
    )
    expect(screen.getByText(/aktivierung fehlgeschlagen/i)).toBeInTheDocument()
  })
})
