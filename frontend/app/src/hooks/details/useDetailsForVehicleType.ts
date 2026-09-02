import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { VehicleType } from '@green-ecolution/backend-client'
import type { EnumsTranslate } from './types'

// Local sentinel for vehicle types not represented by the backend enum.
export const UNKNOWN_VEHICLE_TYPE = 'unknown' as const
export type VehicleTypeOrUnknown = VehicleType | typeof UNKNOWN_VEHICLE_TYPE

const VehicleTypeValues: VehicleTypeOrUnknown[] = [
  VehicleType.Trailer,
  VehicleType.Transporter,
  UNKNOWN_VEHICLE_TYPE,
]

/** Reactive to language change: re-renders whichever component calls it. */
export const useVehicleTypeLabel = (): ((type: VehicleTypeOrUnknown) => string) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (type: VehicleTypeOrUnknown) => translate(`vehicleType.${type}.label`),
    [translate],
  )
}

/** The full type list with translated labels, in display order. */
export const useVehicleTypeOptions = (): { value: VehicleTypeOrUnknown; label: string }[] => {
  const getLabel = useVehicleTypeLabel()
  return useMemo(
    () => VehicleTypeValues.map((value) => ({ value, label: getLabel(value) })),
    [getLabel],
  )
}
