import { VehicleType } from '@green-ecolution/backend-client'
import { createEnumLookup } from '@/lib/enumLookup'

// Local sentinel for vehicle types not represented by the backend enum.
export const UNKNOWN_VEHICLE_TYPE = 'unknown' as const
export type VehicleTypeOrUnknown = VehicleType | typeof UNKNOWN_VEHICLE_TYPE

const UNKNOWN_VEHICLE_TYPE_OPTION: { value: VehicleTypeOrUnknown; label: string } = {
  value: UNKNOWN_VEHICLE_TYPE,
  label: 'Unbekannt',
}

export const VehicleTypeOptions: { value: VehicleTypeOrUnknown; label: string }[] = [
  {
    value: VehicleType.Trailer,
    label: 'Anhänger',
  },
  {
    value: VehicleType.Transporter,
    label: 'Transporter',
  },
  UNKNOWN_VEHICLE_TYPE_OPTION,
]

const getVehicleTypeDetails = createEnumLookup(VehicleTypeOptions, UNKNOWN_VEHICLE_TYPE_OPTION)

export const getVehicleType = (vehicleType: VehicleTypeOrUnknown) =>
  getVehicleTypeDetails(vehicleType).label
