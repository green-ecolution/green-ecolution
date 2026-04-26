import { describe, it, expect } from 'vitest'
import {
  DrivingLicense,
  VehicleStatus,
  VehicleType,
  UserStatus,
  UserRole,
} from '@green-ecolution/backend-client'
import type { User, Vehicle } from '@/api/backendApi'
import { licenseSatisfies, validateDriverLicenses } from './licenseValidation'

describe('licenseSatisfies', () => {
  // B satisfies only B
  it.each([
    [DrivingLicense.B, DrivingLicense.B, true],
    [DrivingLicense.B, DrivingLicense.Be, false],
    [DrivingLicense.B, DrivingLicense.C, false],
    [DrivingLicense.B, DrivingLicense.Ce, false],

    // BE satisfies B, BE
    [DrivingLicense.Be, DrivingLicense.B, true],
    [DrivingLicense.Be, DrivingLicense.Be, true],
    [DrivingLicense.Be, DrivingLicense.C, false],
    [DrivingLicense.Be, DrivingLicense.Ce, false],

    // C satisfies B, C
    [DrivingLicense.C, DrivingLicense.B, true],
    [DrivingLicense.C, DrivingLicense.C, true],
    [DrivingLicense.C, DrivingLicense.Be, false],
    [DrivingLicense.C, DrivingLicense.Ce, false],

    // CE satisfies B, BE, C, CE
    [DrivingLicense.Ce, DrivingLicense.B, true],
    [DrivingLicense.Ce, DrivingLicense.Be, true],
    [DrivingLicense.Ce, DrivingLicense.C, true],
    [DrivingLicense.Ce, DrivingLicense.Ce, true],
  ])('held=%s required=%s → %s', (held, required, expected) => {
    expect(licenseSatisfies(held, required)).toBe(expected)
  })
})

const makeUser = (id: string, licenses: DrivingLicense[]): User => ({
  id,
  firstName: 'Test',
  lastName: 'User',
  email: 'test@example.com',
  emailVerified: true,
  employeeId: '1',
  username: 'testuser',
  phoneNumber: '',
  avatarUrl: '',
  createdAt: '',
  drivingLicenses: licenses,
  roles: [UserRole.Tbz],
  status: UserStatus.Available,
})

const makeVehicle = (id: number, license: DrivingLicense, type: VehicleType): Vehicle => ({
  id,
  numberPlate: `HH-${id}`,
  drivingLicense: license,
  type,
  model: 'Test',
  description: '',
  height: 0,
  length: 0,
  width: 0,
  weight: 0,
  waterCapacity: 0,
  status: VehicleStatus.Active,
  createdAt: '',
  updatedAt: '',
})

describe('validateDriverLicenses', () => {
  const transporters = [
    makeVehicle(1, DrivingLicense.B, VehicleType.Transporter),
    makeVehicle(2, DrivingLicense.C, VehicleType.Transporter),
  ]
  const trailers = [
    makeVehicle(10, DrivingLicense.Be, VehicleType.Trailer),
  ]

  it('returns valid when no drivers selected', () => {
    const result = validateDriverLicenses([], [], transporters, trailers, 1)
    expect(result.valid).toBe(true)
  })

  it('returns valid when no vehicle selected', () => {
    const users = [makeUser('u1', [DrivingLicense.B])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers)
    expect(result.valid).toBe(true)
  })

  it('returns valid when driver has matching license', () => {
    const users = [makeUser('u1', [DrivingLicense.C])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2)
    expect(result.valid).toBe(true)
  })

  it('returns invalid when driver lacks required license', () => {
    const users = [makeUser('u1', [DrivingLicense.B])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2)
    expect(result.valid).toBe(false)
    expect(result.message).toBeDefined()
  })

  it('uses hierarchy: C driver satisfies B vehicle', () => {
    const users = [makeUser('u1', [DrivingLicense.C])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 1)
    expect(result.valid).toBe(true)
  })

  it('returns valid when at least one driver qualifies', () => {
    const users = [
      makeUser('u1', [DrivingLicense.B]),
      makeUser('u2', [DrivingLicense.C]),
    ]
    const result = validateDriverLicenses(['u1', 'u2'], users, transporters, trailers, 2)
    expect(result.valid).toBe(true)
  })

  it('validates transporter + trailer combination', () => {
    const users = [makeUser('u1', [DrivingLicense.Ce])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2, 10)
    expect(result.valid).toBe(true)
  })

  it('fails transporter + trailer when no driver qualifies for both', () => {
    const users = [makeUser('u1', [DrivingLicense.C])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2, 10)
    expect(result.valid).toBe(false)
  })
})
