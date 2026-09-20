import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { DataHealth, SensorStatus, SensorTypeResponse } from '@green-ecolution/backend-client'
import SensorCard from './SensorCard'
import type { Sensor } from '@/api/backendApi'

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

  it('names the watering group of the linked tree', () => {
    render(
      <SensorCard
        sensor={sensorFixture({ linkedTreeId: 'tree-1', linkedClusterName: 'Hafenspitze' })}
        query=""
      />,
    )

    expect(screen.getByText('Hafenspitze')).toBeInTheDocument()
  })

  it('marks the search match in the group name', () => {
    render(
      <SensorCard
        sensor={sensorFixture({ linkedTreeId: 'tree-1', linkedClusterName: 'Hafenspitze' })}
        query="hafen"
      />,
    )

    expect(screen.getByText('Hafen').tagName).toBe('MARK')
  })

  it('omits the group line for a sensor whose tree is in no group', () => {
    render(<SensorCard sensor={sensorFixture({ linkedTreeId: 'tree-1' })} query="" />)

    expect(screen.queryByText(/Bewässerungsgruppe/)).not.toBeInTheDocument()
  })

  it('shows a quality warning badge for a sensor under data-quality suspicion', () => {
    render(<SensorCard sensor={sensorFixture({ dataHealth: DataHealth.Suspect })} query="" />)

    expect(screen.getByText(/Datenqualität prüfen/i)).toBeInTheDocument()
  })
})
