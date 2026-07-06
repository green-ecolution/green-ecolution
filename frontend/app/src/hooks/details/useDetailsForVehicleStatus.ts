import { VehicleStatus } from '@green-ecolution/backend-client'
import { createEnumLookup } from '@/lib/enumLookup'
import { StatusColor } from './types'

export const VehicleStatusOptions: {
  value: VehicleStatus
  color: StatusColor
  bgcolor: string
  label: string
  description: string
}[] = [
  {
    value: VehicleStatus.Unknown,
    color: 'outline-dark',
    bgcolor: 'none',
    label: 'Unbekannt',
    description: 'Der Fahrzeugstatus ist unbekannt.',
  },
  {
    value: VehicleStatus.NotAvailable,
    color: 'outline-red',
    bgcolor: 'none',
    label: 'Nicht Verfügbar',
    description: 'Das Fahrzeug ist nicht verfügbar.',
  },
  {
    value: VehicleStatus.Available,
    color: 'outline-green-dark',
    bgcolor: 'none',
    label: 'Verfügbar',
    description: 'Das Fahrzeug ist verfügbar.',
  },
  {
    value: VehicleStatus.Active,
    color: 'outline-green-light',
    bgcolor: 'green-light-200',
    label: 'Im Einsatz',
    description: 'Das Fahrzeug ist im Einsatz.',
  },
]

export const getVehicleStatusDetails = createEnumLookup(VehicleStatusOptions)
