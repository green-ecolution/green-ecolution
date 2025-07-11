import { VehicleType } from '@green-ecolution/backend-client'

export const VehicleTypeOptions = [
  {
    value: VehicleType.VehicleTypeTrailer,
    label: 'Anhänger',
  },
  {
    value: VehicleType.VehicleTypeTransporter,
    label: 'Transporter',
  },
  {
    value: VehicleType.VehicleTypeUnknown,
    label: 'Unbekannt',
  },
]

export const getVehicleType = (vehicleType: VehicleType) => {
  const match = VehicleTypeOptions.find((option) => option.value === vehicleType)
  return match ? match.label : 'Unbekannt'
}
