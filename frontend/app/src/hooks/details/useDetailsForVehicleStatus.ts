import { VehicleStatus } from '@green-ecolution/backend-client'
import { StatusColor } from './useDetailsForWateringPlanStatus'

export const VehicleStatusOptions: {
  value: VehicleStatus
  color: StatusColor
  bgcolor: string
  label: string
  description: string
}[] = [
  {
    value: VehicleStatus.VehicleStatusUnknown,
    color: 'dark-400',
    bgcolor: 'none',
    label: 'Unbekannt',
    description: 'Der Fahrzeugstatus ist unbekannt.',
  },
  {
    value: VehicleStatus.VehicleStatusNotAvailable,
    color: 'red',
    bgcolor: 'none',
    label: 'Nicht Verfügbar',
    description: 'Das Fahrzeug ist nicht verfügbar.',
  },
  {
    value: VehicleStatus.VehicleStatusAvailable,
    color: 'green-dark',
    bgcolor: 'none',
    label: 'Verfügbar',
    description: 'Das Fahrzeug ist verfügbar.',
  },
  {
    value: VehicleStatus.VehicleStatusActive,
    color: 'green-light',
    bgcolor: 'green-light-200',
    label: 'Im Einsatz',
    description: 'Das Fahrzeug ist im Einsatz.',
  },
]

export const getVehicleStatusDetails = (status: VehicleStatus) =>
  VehicleStatusOptions.find((option) => option.value === status) ?? VehicleStatusOptions[0]
