import { DrivingLicense, User, Vehicle } from '@green-ecolution/backend-client'

export function licenseSatisfies(held: DrivingLicense, required: DrivingLicense): boolean {
  if (held === required) {
    return true
  }
  switch (held) {
    case DrivingLicense.DrivingLicenseBE:
      return required === DrivingLicense.DrivingLicenseB
    case DrivingLicense.DrivingLicenseC:
      return required === DrivingLicense.DrivingLicenseB
    case DrivingLicense.DrivingLicenseCE:
      return (
        required === DrivingLicense.DrivingLicenseB ||
        required === DrivingLicense.DrivingLicenseBE ||
        required === DrivingLicense.DrivingLicenseC
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
  transporterId?: number,
  trailerId?: number,
): { valid: boolean; message?: string } {
  if (!driverIds || driverIds.length === 0) {
    return { valid: true }
  }

  const requiredLicenses: DrivingLicense[] = []

  if (transporterId && transporterId > 0) {
    const transporter = transporters.find((t) => t.id === transporterId)
    if (transporter) {
      requiredLicenses.push(transporter.drivingLicense)
    }
  }

  if (trailerId && trailerId > 0) {
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
    requiredLicenses.every((required) =>
      user.drivingLicenses.some((held) => licenseSatisfies(held, required)),
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
