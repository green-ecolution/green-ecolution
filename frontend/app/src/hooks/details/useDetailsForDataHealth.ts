import { DataHealth } from '@green-ecolution/backend-client'
import type { StatusColor } from './types'

const DataHealthProperties: Record<
  DataHealth,
  { color: StatusColor; label: string; description: string }
> = {
  [DataHealth.Ok]: {
    color: 'outline-green-dark',
    label: 'Daten plausibel',
    description: 'Die zuletzt empfangenen Messwerte liegen im erwarteten Bereich.',
  },
  [DataHealth.Suspect]: {
    color: 'outline-red',
    label: 'Datenqualität prüfen',
    description:
      'Mehrere Übertragungen in Folge enthielten keinen verwertbaren Messwert. Der Sensor ist womöglich defekt.',
  },
}

export const getDataHealthDetails = (health: DataHealth) => DataHealthProperties[health]

export const hasQualityWarning = (sensor: {
  dataHealth: DataHealth
  implausibleRecent: number
}): boolean => sensor.dataHealth === DataHealth.Suspect || sensor.implausibleRecent > 0

const QUALITY_REASON_LABEL: Record<string, string> = {
  out_of_range: 'Wert außerhalb des möglichen Bereichs',
  implausible_jump: 'Unplausibler Sprung gegenüber dem Vorwert',
}

export const qualityReasonLabel = (reason: string): string =>
  QUALITY_REASON_LABEL[reason] ?? 'Unbekannter Grund'
