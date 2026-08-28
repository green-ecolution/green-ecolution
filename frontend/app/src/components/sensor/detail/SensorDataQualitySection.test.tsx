import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { DataHealth } from '@green-ecolution/backend-client'
import SensorDataQualitySection from './SensorDataQualitySection'

const { useQueryMock } = vi.hoisted(() => ({ useQueryMock: vi.fn() }))

vi.mock('@tanstack/react-query', () => ({
  useQuery: useQueryMock,
  queryOptions: (options: unknown) => options,
}))

describe('SensorDataQualitySection', () => {
  it('renders the flagged measurement with a readable reason', () => {
    useQueryMock.mockReturnValue({
      data: {
        health: DataHealth.Suspect,
        implausibleRecent: 2,
        issues: [
          {
            recordedAt: '2026-08-20T06:15:00+00:00',
            ability: 'soil_moisture',
            depthCm: 40,
            value: 6553.5,
            reason: 'out_of_range',
          },
        ],
      },
    })

    render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(screen.getByText(/Wert außerhalb des möglichen Bereichs/)).toBeInTheDocument()
    expect(screen.getByText(/40 cm Tiefe/)).toBeInTheDocument()
    expect(screen.getByText('Datenqualität prüfen')).toBeInTheDocument()
  })

  it('does not claim a seven-day count when the flagged values are older', () => {
    useQueryMock.mockReturnValue({
      data: {
        health: DataHealth.Ok,
        implausibleRecent: 0,
        issues: [
          {
            recordedAt: '2026-08-01T06:15:00+00:00',
            ability: 'soil_moisture',
            depthCm: 40,
            value: 6553.5,
            reason: 'out_of_range',
          },
        ],
      },
    })

    render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(screen.getByText(/länger als sieben Tage zurück/)).toBeInTheDocument()
    expect(screen.queryByText(/wurden 0 Messwerte/)).not.toBeInTheDocument()
  })

  it('renders nothing when no value was flagged', () => {
    useQueryMock.mockReturnValue({
      data: { health: DataHealth.Ok, implausibleRecent: 0, issues: [] },
    })

    const { container } = render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(container).toBeEmptyDOMElement()
  })
})
