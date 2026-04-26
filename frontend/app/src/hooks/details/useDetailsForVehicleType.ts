import { VehicleType } from '@green-ecolution/backend-client'

export const VehicleTypeOptions = [
  {
    value: VehicleType.Trailer,
    label: 'Anhänger',
  },
  {
    value: VehicleType.Transporter,
    label: 'Transporter',
  },
  {
    value: VehicleType.Unknown,
    label: 'Unbekannt',
  },
]

export const getVehicleType = (vehicleType: VehicleType) => {
  const match = VehicleTypeOptions.find((option) => option.value === vehicleType)
  return match ? match.label : 'Unbekannt'
}
