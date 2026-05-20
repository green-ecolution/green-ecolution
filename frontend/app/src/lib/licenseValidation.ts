import { DrivingLicense } from '@green-ecolution/backend-client'
import type { User, Vehicle } from '@/api/backendApi'

// Keep in sync with backend: internal/service/domain/watering_plan/watering_plan.go
export function licenseSatisfies(held: DrivingLicense, required: DrivingLicense): boolean {
  if (held === required) {
    return true
  }
  switch (held) {
    case DrivingLicense.Be:
      return required === DrivingLicense.B
    case DrivingLicense.C:
      return required === DrivingLicense.B
    case DrivingLicense.Ce:
      return (
        required === DrivingLicense.B ||
        required === DrivingLicense.Be ||
        required === DrivingLicense.C
      )
    default:
      return false
  }
}

export function validateDriverLicenses(
  driverIds: string[],
  users: User[],
  transporters: Vehicle[],
  trailers: Vehicle[],
  transporterId?: string,
  trailerId?: string,
): { valid: boolean; message?: string } {
  if (!driverIds || driverIds.length === 0) {
    return { valid: true }
  }

  const requiredLicenses: DrivingLicense[] = []

  if (transporterId !== undefined && transporterId !== '' && transporterId !== '-1') {
    const transporter = transporters.find((t) => t.id === transporterId)
    if (transporter) {
      requiredLicenses.push(transporter.drivingLicense)
    }
  }

  if (trailerId !== undefined && trailerId !== '' && trailerId !== '-1') {
    const trailer = trailers.find((t) => t.id === trailerId)
    if (trailer) {
      requiredLicenses.push(trailer.drivingLicense)
    }
  }

  if (requiredLicenses.length === 0) {
    return { valid: true }
  }

  const selectedUsers = users.filter((u) => driverIds.includes(u.id))
  const hasQualifiedDriver = selectedUsers.some((user) =>
    requiredLicenses.every((required: DrivingLicense) =>
      user.drivingLicenses.some((held: DrivingLicense) => licenseSatisfies(held, required)),
    ),
  )

  if (!hasQualifiedDriver) {
    return {
      valid: false,
      message:
        'Kein ausgewählter Mitarbeiter hat alle erforderlichen Führerscheine für die gewählten Fahrzeuge.',
    }
  }

  return { valid: true }
}
