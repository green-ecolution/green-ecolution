import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { DataHealth, SensorStatus, SensorTypeResponse } from '@green-ecolution/backend-client'
import SensorCard from './SensorCard'
import type { Sensor } from '@/api/backendApi'

const target = { module: 'probe', kind: 'probe', id: 'p-1' }

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => <a href="/">{children}</a>,
}))

const sensorFixture = (overrides: Partial<Sensor> = {}): Sensor => ({
  id: 'eui-a81758fffe0c3b52',
  organizationId: 'org-1',
  sensorType: SensorTypeResponse.Lorawan,
  status: SensorStatus.Online,
  dataHealth: DataHealth.Ok,
  implausibleRecent: 0,
  model: { id: 'model-1', name: 'TEROS 12', abilities: [] },
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  ...overrides,
})

describe('SensorCard', () => {
  it('marks the search match in the EUI', () => {
    render(<SensorCard sensor={sensorFixture({ id: 'eui-a81758fffe0c3b52' })} query="a81758" />)

    expect(screen.getByText('a81758').tagName).toBe('MARK')
  })

  it('names the placement the module reports for the target', () => {
    render(<SensorCard sensor={sensorFixture({ target, targetLabel: 'Hafenspitze' })} query="" />)

    expect(screen.getByText('Hafenspitze')).toBeInTheDocument()
  })

  it('marks the search match in the placement name', () => {
    render(
      <SensorCard sensor={sensorFixture({ target, targetLabel: 'Hafenspitze' })} query="hafen" />,
    )

    expect(screen.getByText('Hafen').tagName).toBe('MARK')
  })

  it('omits the placement line when the module reports no label', () => {
    render(<SensorCard sensor={sensorFixture({ target, targetLabel: null })} query="" />)

    expect(screen.queryByText(/Einsatzort/)).not.toBeInTheDocument()
  })

  it('says whether the sensor is linked', () => {
    const { rerender } = render(<SensorCard sensor={sensorFixture({ target })} query="" />)
    expect(screen.getByText('Verknüpft')).toBeInTheDocument()

    rerender(<SensorCard sensor={sensorFixture({ target: null })} query="" />)
    expect(screen.getByText('Nicht verknüpft')).toBeInTheDocument()
  })

  it('shows a quality warning badge for a sensor under data-quality suspicion', () => {
    render(<SensorCard sensor={sensorFixture({ dataHealth: DataHealth.Suspect })} query="" />)

    expect(screen.getByText(/Datenqualität prüfen/i)).toBeInTheDocument()
  })
})
