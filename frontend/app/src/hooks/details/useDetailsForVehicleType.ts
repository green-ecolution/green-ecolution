import { VehicleType } from '@green-ecolution/backend-client'

// Local sentinel for vehicle types not represented by the backend enum.
export const UNKNOWN_VEHICLE_TYPE = 'unknown' as const
export type VehicleTypeOrUnknown = VehicleType | typeof UNKNOWN_VEHICLE_TYPE

export const VehicleTypeOptions: { value: VehicleTypeOrUnknown; label: string }[] = [
  {
    value: VehicleType.Trailer,
    label: 'Anhänger',
  },
  {
    value: VehicleType.Transporter,
    label: 'Transporter',
  },
  {
    value: UNKNOWN_VEHICLE_TYPE,
    label: 'Unbekannt',
  },
]

export const getVehicleType = (vehicleType: VehicleTypeOrUnknown) => {
  const match = VehicleTypeOptions.find((option) => option.value === vehicleType)
  return match ? match.label : 'Unbekannt'
}
