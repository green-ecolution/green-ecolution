import { DrivingLicense } from '@green-ecolution/backend-client'
import { licenseSatisfies as wasmLicenseSatisfies } from '@green-ecolution/domain-wasm'
import type { User, Vehicle } from '@/api/backendApi'

// Single source of truth lives in the Rust domain (DrivingLicense::satisfies),
// shared via WASM. No TS-side hierarchy logic remains, so it cannot drift.
export function licenseSatisfies(held: DrivingLicense, required: DrivingLicense): boolean {
  return wasmLicenseSatisfies(held, required)
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
