import { SensorStatus } from '@green-ecolution/backend-client'

const SensorStatusProperties = {
  [SensorStatus.SensorStatusUnknown]: {
    color: 'dark-400',
    label: 'Unbekannt',
    description: 'Der Status ist unbekannt.',
  },
  [SensorStatus.SensorStatusOffline]: {
    color: 'red',
    label: 'Offline',
    description: 'Der Sensorbaukasten hat Probleme und benötigen eine Wartung.',
  },
  [SensorStatus.SensorStatusOnline]: {
    color: 'green-dark',
    label: 'Online',
    description: 'Der Sensorbaukasten kann Daten senden.',
  },
} as const

type SensorStatusDetails = (typeof SensorStatusProperties)[SensorStatus]

export const getSensorStatusDetails = (status: SensorStatus): SensorStatusDetails => {
  return SensorStatusProperties[status]
}
