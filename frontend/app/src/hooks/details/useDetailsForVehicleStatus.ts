import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { VehicleStatus } from '@green-ecolution/backend-client'
import { EnumsTranslate, StatusColor } from './types'

const VehicleStatusEntries: { value: VehicleStatus; color: StatusColor; bgcolor: string }[] = [
  { value: VehicleStatus.Unknown, color: 'outline-dark', bgcolor: 'none' },
  { value: VehicleStatus.NotAvailable, color: 'outline-red', bgcolor: 'none' },
  { value: VehicleStatus.Available, color: 'outline-green-dark', bgcolor: 'none' },
  { value: VehicleStatus.Active, color: 'outline-green-light', bgcolor: 'green-light-200' },
]

export interface VehicleStatusDetails {
  value: VehicleStatus
  color: StatusColor
  bgcolor: string
  label: string
  description: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useVehicleStatusDetails = (): ((status: VehicleStatus) => VehicleStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: VehicleStatus): VehicleStatusDetails => {
      const entry =
        VehicleStatusEntries.find((option) => option.value === status) ?? VehicleStatusEntries[0]
      return {
        ...entry,
        label: translate(`vehicleStatus.${entry.value}.label`),
        description: translate(`vehicleStatus.${entry.value}.description`),
      }
    },
    [translate],
  )
}

/** The full status list with translated labels, in display order. */
export const useVehicleStatusOptions = (): VehicleStatusDetails[] => {
  const getDetails = useVehicleStatusDetails()
  return useMemo(
    () => VehicleStatusEntries.map((entry) => getDetails(entry.value)),
    [getDetails],
  )
}
