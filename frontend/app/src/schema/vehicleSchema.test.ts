import { describe, it, expect } from 'vitest'
import { vehicleSchema } from './vehicleSchema'
import { VehicleType, DrivingLicense, VehicleStatus } from '@green-ecolution/backend-client'

describe('vehicleSchema', () => {
  const validVehicle = {
    numberPlate: 'HH-AB-1234',
    model: 'Mercedes Sprinter',
    type: VehicleType.VehicleTypeTransporter,
    drivingLicense: DrivingLicense.DrivingLicenseB,
    status: VehicleStatus.VehicleStatusAvailable,
    height: 2.5,
    width: 2.0,
    length: 6.0,
    weight: 3.5,
    waterCapacity: 1000,
    description: 'Test vehicle',
  }

  it('validates a complete valid vehicle object', () => {
    const result = vehicleSchema.safeParse(validVehicle)
    expect(result.success).toBe(true)
  })

  it('requires non-empty numberPlate', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, numberPlate: '' })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Kennzeichen')
    }
  })

  it('requires non-empty model', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, model: '' })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('Modell')
    }
  })

  it('accepts all valid VehicleType values', () => {
    const vehicleTypes = [
      VehicleType.VehicleTypeTransporter,
      VehicleType.VehicleTypeTrailer,
      VehicleType.VehicleTypeUnknown,
    ]

    vehicleTypes.forEach((type) => {
      const result = vehicleSchema.safeParse({ ...validVehicle, type })
      expect(result.success).toBe(true)
    })
  })

  it('rejects invalid VehicleType value', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, type: 'invalid' })
    expect(result.success).toBe(false)
  })

  it('accepts all valid DrivingLicense values', () => {
    const licenses = [
      DrivingLicense.DrivingLicenseB,
      DrivingLicense.DrivingLicenseBE,
      DrivingLicense.DrivingLicenseC,
      DrivingLicense.DrivingLicenseCE,
    ]

    licenses.forEach((drivingLicense) => {
      const result = vehicleSchema.safeParse({ ...validVehicle, drivingLicense })
      expect(result.success).toBe(true)
    })
  })

  it('rejects invalid DrivingLicense value', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, drivingLicense: 'invalid' })
    expect(result.success).toBe(false)
  })

  it('accepts all valid VehicleStatus values', () => {
    const statuses = [
      VehicleStatus.VehicleStatusActive,
      VehicleStatus.VehicleStatusAvailable,
      VehicleStatus.VehicleStatusNotAvailable,
      VehicleStatus.VehicleStatusUnknown,
    ]

    statuses.forEach((status) => {
      const result = vehicleSchema.safeParse({ ...validVehicle, status })
      expect(result.success).toBe(true)
    })
  })

  it('rejects invalid VehicleStatus value', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, status: 'invalid' })
    expect(result.success).toBe(false)
  })

  it('requires positive height', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, height: 0 })
    expect(result.success).toBe(false)
  })

  it('requires positive width', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, width: 0 })
    expect(result.success).toBe(false)
  })

  it('requires positive length', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, length: 0 })
    expect(result.success).toBe(false)
  })

  it('requires positive weight', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, weight: 0 })
    expect(result.success).toBe(false)
  })

  it('rejects waterCapacity below 80', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, waterCapacity: 79 })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.issues[0].message).toContain('80')
    }
  })

  it('accepts waterCapacity of exactly 80', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, waterCapacity: 80 })
    expect(result.success).toBe(true)
  })

  it('accepts waterCapacity above 80', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, waterCapacity: 5000 })
    expect(result.success).toBe(true)
  })

  it('allows empty description', () => {
    const result = vehicleSchema.safeParse({ ...validVehicle, description: '' })
    expect(result.success).toBe(true)
  })

  it('allows description with content', () => {
    const result = vehicleSchema.safeParse({
      ...validVehicle,
      description: 'This is a detailed vehicle description',
    })
    expect(result.success).toBe(true)
  })

  it('coerces string numbers to numbers', () => {
    const result = vehicleSchema.safeParse({
      ...validVehicle,
      height: '2.5',
      width: '2.0',
      length: '6.0',
      weight: '3.5',
      waterCapacity: '1000',
    })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.data.height).toBe(2.5)
      expect(result.data.waterCapacity).toBe(1000)
    }
  })
})
