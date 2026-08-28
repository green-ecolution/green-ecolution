import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { DataHealth } from '@green-ecolution/backend-client'
import SensorDataQualitySection from './SensorDataQualitySection'

const { useQueryMock, acknowledgeMock } = vi.hoisted(() => ({
  useQueryMock: vi.fn(),
  acknowledgeMock: { mutate: vi.fn(), isPending: false },
}))

vi.mock('@tanstack/react-query', () => ({
  useQuery: useQueryMock,
  queryOptions: (options: unknown) => options,
}))

vi.mock('@/hooks/useSensorQualityMutations', () => ({
  useSensorQualityMutations: () => ({ acknowledge: acknowledgeMock }),
}))

vi.mock('@/lib/auth/Can', () => ({
  Can: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

const issue = (recordedAt: string) => ({
  recordedAt,
  ability: 'soil_moisture',
  depthCm: 40,
  value: 6553.5,
  reason: 'out_of_range',
})

describe('SensorDataQualitySection', () => {
  it('renders the flagged measurement with a readable reason', () => {
    useQueryMock.mockReturnValue({
      data: {
        health: DataHealth.Suspect,
        implausibleRecent: 2,
        issues: [issue('2026-08-20T06:15:00+00:00')],
      },
    })

    render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(screen.getByText(/Wert außerhalb des möglichen Bereichs/)).toBeInTheDocument()
    expect(screen.getByText(/40 cm Tiefe/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Zur Kenntnis genommen' })).toBeInTheDocument()
  })

  it('moves acknowledged values into the collapsed history and drops the button', () => {
    useQueryMock.mockReturnValue({
      data: {
        health: DataHealth.Ok,
        implausibleRecent: 0,
        issues: [issue('2026-08-20T06:15:00+00:00')],
        acknowledged: {
          at: '2026-08-21T09:00:00+00:00',
          byId: 'user-1',
          byName: 'Cedrik Hoffmann',
          note: 'Sonde war nicht angeschlossen',
        },
      },
    })

    render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(screen.getByText(/Frühere Auffälligkeiten \(1\)/)).toBeInTheDocument()
    expect(screen.getByText(/Sonde war nicht angeschlossen/)).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Zur Kenntnis genommen' })).not.toBeInTheDocument()
  })

  it('offers the button again for values flagged after the acknowledgement', () => {
    useQueryMock.mockReturnValue({
      data: {
        health: DataHealth.Ok,
        implausibleRecent: 1,
        issues: [issue('2026-08-22T06:15:00+00:00'), issue('2026-08-20T06:15:00+00:00')],
        acknowledged: {
          at: '2026-08-21T09:00:00+00:00',
          byId: 'user-1',
          byName: 'Cedrik Hoffmann',
          note: null,
        },
      },
    })

    render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(screen.getByRole('button', { name: 'Zur Kenntnis genommen' })).toBeInTheDocument()
    expect(screen.getByText(/Frühere Auffälligkeiten \(1\)/)).toBeInTheDocument()
  })

  it('renders nothing when no value was flagged', () => {
    useQueryMock.mockReturnValue({
      data: { health: DataHealth.Ok, implausibleRecent: 0, issues: [] },
    })

    const { container } = render(<SensorDataQualitySection sensorId="eui-test" />)

    expect(container).toBeEmptyDOMElement()
  })
})
