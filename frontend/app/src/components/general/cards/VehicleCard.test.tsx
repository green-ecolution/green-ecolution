import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import {
  DrivingLicense,
  VehicleAvailability,
  VehicleStatus,
  VehicleType,
} from '@green-ecolution/backend-client'
import VehicleCard from './VehicleCard'
import type { Vehicle } from '@/api/backendApi'

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
  waterCapacity: 1000,
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
})
