import { describe, it, expect } from 'vitest'
import {
  DrivingLicense,
  User,
  Vehicle,
  VehicleStatus,
  VehicleType,
  UserStatus,
  UserRole,
} from '@green-ecolution/backend-client'
import { licenseSatisfies, validateDriverLicenses } from './useLicenseValidation'

describe('licenseSatisfies', () => {
  // B satisfies only B
  it.each([
    [DrivingLicense.DrivingLicenseB, DrivingLicense.DrivingLicenseB, true],
    [DrivingLicense.DrivingLicenseB, DrivingLicense.DrivingLicenseBE, false],
    [DrivingLicense.DrivingLicenseB, DrivingLicense.DrivingLicenseC, false],
    [DrivingLicense.DrivingLicenseB, DrivingLicense.DrivingLicenseCE, false],

    // BE satisfies B, BE
    [DrivingLicense.DrivingLicenseBE, DrivingLicense.DrivingLicenseB, true],
    [DrivingLicense.DrivingLicenseBE, DrivingLicense.DrivingLicenseBE, true],
    [DrivingLicense.DrivingLicenseBE, DrivingLicense.DrivingLicenseC, false],
    [DrivingLicense.DrivingLicenseBE, DrivingLicense.DrivingLicenseCE, false],

    // C satisfies B, C
    [DrivingLicense.DrivingLicenseC, DrivingLicense.DrivingLicenseB, true],
    [DrivingLicense.DrivingLicenseC, DrivingLicense.DrivingLicenseC, true],
    [DrivingLicense.DrivingLicenseC, DrivingLicense.DrivingLicenseBE, false],
    [DrivingLicense.DrivingLicenseC, DrivingLicense.DrivingLicenseCE, false],

    // CE satisfies B, BE, C, CE
    [DrivingLicense.DrivingLicenseCE, DrivingLicense.DrivingLicenseB, true],
    [DrivingLicense.DrivingLicenseCE, DrivingLicense.DrivingLicenseBE, true],
    [DrivingLicense.DrivingLicenseCE, DrivingLicense.DrivingLicenseC, true],
    [DrivingLicense.DrivingLicenseCE, DrivingLicense.DrivingLicenseCE, true],
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
  roles: [UserRole.UserRoleTbz],
  status: UserStatus.UserStatusAvailable,
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
  status: VehicleStatus.VehicleStatusActive,
  createdAt: '',
  updatedAt: '',
})

describe('validateDriverLicenses', () => {
  const transporters = [
    makeVehicle(1, DrivingLicense.DrivingLicenseB, VehicleType.VehicleTypeTransporter),
    makeVehicle(2, DrivingLicense.DrivingLicenseC, VehicleType.VehicleTypeTransporter),
  ]
  const trailers = [
    makeVehicle(10, DrivingLicense.DrivingLicenseBE, VehicleType.VehicleTypeTrailer),
  ]

  it('returns valid when no drivers selected', () => {
    const result = validateDriverLicenses([], [], transporters, trailers, 1)
    expect(result.valid).toBe(true)
  })

  it('returns valid when no vehicle selected', () => {
    const users = [makeUser('u1', [DrivingLicense.DrivingLicenseB])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers)
    expect(result.valid).toBe(true)
  })

  it('returns valid when driver has matching license', () => {
    const users = [makeUser('u1', [DrivingLicense.DrivingLicenseC])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2)
    expect(result.valid).toBe(true)
  })

  it('returns invalid when driver lacks required license', () => {
    const users = [makeUser('u1', [DrivingLicense.DrivingLicenseB])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2)
    expect(result.valid).toBe(false)
    expect(result.message).toBeDefined()
  })

  it('uses hierarchy: C driver satisfies B vehicle', () => {
    const users = [makeUser('u1', [DrivingLicense.DrivingLicenseC])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 1)
    expect(result.valid).toBe(true)
  })

  it('returns valid when at least one driver qualifies', () => {
    const users = [
      makeUser('u1', [DrivingLicense.DrivingLicenseB]),
      makeUser('u2', [DrivingLicense.DrivingLicenseC]),
    ]
    const result = validateDriverLicenses(['u1', 'u2'], users, transporters, trailers, 2)
    expect(result.valid).toBe(true)
  })

  it('validates transporter + trailer combination', () => {
    const users = [makeUser('u1', [DrivingLicense.DrivingLicenseCE])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2, 10)
    expect(result.valid).toBe(true)
  })

  it('fails transporter + trailer when no driver qualifies for both', () => {
    const users = [makeUser('u1', [DrivingLicense.DrivingLicenseC])]
    const result = validateDriverLicenses(['u1'], users, transporters, trailers, 2, 10)
    expect(result.valid).toBe(false)
  })
})
