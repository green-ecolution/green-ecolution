import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { VehicleAvailability, VehicleStatus } from '@green-ecolution/backend-client'
import { EnumsTranslate, StatusColor } from './types'

interface StatusStyle {
  color: StatusColor
  bgcolor: string
}

const VehicleStatusStyles: Record<VehicleStatus, StatusStyle> = {
  [VehicleStatus.NotAvailable]: { color: 'outline-red', bgcolor: 'none' },
  [VehicleStatus.Available]: { color: 'outline-green-dark', bgcolor: 'none' },
  [VehicleStatus.Active]: { color: 'outline-green-light', bgcolor: 'green-light-200' },
}

/** Only a human can assert these two; `active` is derived and never chosen. */
const VehicleAvailabilityOrder: VehicleAvailability[] = [
  VehicleAvailability.Available,
  VehicleAvailability.NotAvailable,
]

export interface VehicleStatusDetails extends StatusStyle {
  value: VehicleStatus
  label: string
  description: string
}

export interface VehicleAvailabilityDetails {
  value: VehicleAvailability
  label: string
  description: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useVehicleStatusDetails = (): ((status: VehicleStatus) => VehicleStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: VehicleStatus): VehicleStatusDetails => ({
      ...VehicleStatusStyles[status],
      value: status,
      label: translate(`vehicleStatus.${status}.label`),
      description: translate(`vehicleStatus.${status}.description`),
    }),
    [translate],
  )
}

/** The selectable availabilities with translated labels, in display order. */
export const useVehicleAvailabilityOptions = (): VehicleAvailabilityDetails[] => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useMemo(
    () =>
      VehicleAvailabilityOrder.map((value) => ({
        value,
        label: translate(`vehicleStatus.${value}.label`),
        description: translate(`vehicleStatus.${value}.description`),
      })),
    [translate],
  )
}
