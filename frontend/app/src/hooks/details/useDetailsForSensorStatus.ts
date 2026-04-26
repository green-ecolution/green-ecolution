import { SensorStatus } from '@green-ecolution/backend-client'
import type { StatusColor } from './useDetailsForWateringPlanStatus'

const SensorStatusProperties: Record<
  SensorStatus,
  { color: StatusColor; label: string; description: string }
> = {
  [SensorStatus.Unknown]: {
    color: 'outline-dark',
    label: 'Unbekannt',
    description: 'Der Status ist unbekannt.',
  },
  [SensorStatus.Offline]: {
    color: 'outline-red',
    label: 'Offline',
    description: 'Der Sensorbaukasten hat Probleme und benötigen eine Wartung.',
  },
  [SensorStatus.Online]: {
    color: 'outline-green-dark',
    label: 'Online',
    description: 'Der Sensorbaukasten kann Daten senden.',
  },
}

type SensorStatusDetails = (typeof SensorStatusProperties)[SensorStatus]

export const getSensorStatusDetails = (status: SensorStatus): SensorStatusDetails => {
  return SensorStatusProperties[status]
}
