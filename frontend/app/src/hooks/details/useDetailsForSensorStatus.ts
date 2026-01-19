import { SensorStatus } from '@green-ecolution/backend-client'
import type { StatusColor } from './useDetailsForWateringPlanStatus'

const SensorStatusProperties: Record<
  SensorStatus,
  { color: StatusColor; label: string; description: string }
> = {
  [SensorStatus.SensorStatusUnknown]: {
    color: 'outline-dark',
    label: 'Unbekannt',
    description: 'Der Status ist unbekannt.',
  },
  [SensorStatus.SensorStatusOffline]: {
    color: 'outline-red',
    label: 'Offline',
    description: 'Der Sensorbaukasten hat Probleme und benötigen eine Wartung.',
  },
  [SensorStatus.SensorStatusOnline]: {
    color: 'outline-green-dark',
    label: 'Online',
    description: 'Der Sensorbaukasten kann Daten senden.',
  },
}

type SensorStatusDetails = (typeof SensorStatusProperties)[SensorStatus]

export const getSensorStatusDetails = (status: SensorStatus): SensorStatusDetails => {
  return SensorStatusProperties[status]
}
