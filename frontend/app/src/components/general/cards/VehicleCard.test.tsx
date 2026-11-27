import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import {
  DrivingLicense,
  VehicleAvailability,
  VehicleStatus,
  VehicleType,
} from '@green-ecolution/backend-client'
import type { Vehicle } from '@/api/backendApi'
import type { ModuleCapacityKind } from '@/modules/types'
import { manifestOf } from '@/test/moduleFixtures'

const water = manifestOf('irrigation').capacities![0]
const cargo = manifestOf('waste').capacities![0]
const offered = vi.fn((): ModuleCapacityKind[] => [water, cargo])
const offeredFor = vi.fn()
vi.mock('@/modules/capacities', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/modules/capacities')>()),
  useCapacityKinds: (organizationId: string | undefined) => {
    offeredFor(organizationId)
    return offered()
  },
}))

const { default: VehicleCard } = await import('./VehicleCard')

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => <a href="/">{children}</a>,
}))

const vehicleFixture = (overrides: Partial<Vehicle> = {}): Vehicle => ({
  id: '1',
  numberPlate: 'FL-GE 123',
  model: 'Iveco Daily',
  type: VehicleType.Transporter,
  status: VehicleStatus.Available,
  availability: VehicleAvailability.Available,
  drivingLicense: DrivingLicense.B,
  capacities: { water: 1000 },
  height: 2.5,
  width: 2,
  length: 5,
  weight: 3.5,
  organizationId: 'org-1',
  description: '',
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  archivedAt: undefined,
  ...overrides,
})

describe('VehicleCard', () => {
  it('marks the search match in the number plate', () => {
    render(<VehicleCard vehicle={vehicleFixture({ numberPlate: 'FL-GE 123' })} query="GE" />)

    expect(screen.getByText('GE').tagName).toBe('MARK')
  })

  it('shows an archived vehicle as archived', () => {
    render(
      <VehicleCard vehicle={vehicleFixture({ archivedAt: '2026-01-01T00:00:00Z' })} query="" />,
    )

    expect(screen.getByText(/archiviert/i)).toBeInTheDocument()
  })

  it('shows the cargo volume and no water row for a vehicle with cargo volume only', () => {
    render(
      <VehicleCard vehicle={vehicleFixture({ capacities: { cargo_volume: 6000 } })} query="" />,
    )

    expect(screen.getByText(/laderaum/i)).toBeInTheDocument()
    expect(screen.queryByText(/wassertank/i)).not.toBeInTheDocument()
    expect(screen.getByText('6000 Liter')).toBeInTheDocument()
  })

  it('shows only the kinds of the modules active in its organization', () => {
    offered.mockReturnValueOnce([cargo])

    render(
      <VehicleCard
        vehicle={vehicleFixture({ capacities: { water: 1000, cargo_volume: 6000, sand: 3 } })}
        query=""
      />,
    )

    expect(offeredFor).toHaveBeenCalledWith('org-1')
    expect(screen.getByText('6000 Liter')).toBeInTheDocument()
    expect(screen.queryByText('1000 Liter')).not.toBeInTheDocument()
    expect(screen.queryByText(/sand/i)).not.toBeInTheDocument()
  })
})
