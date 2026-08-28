import { DataHealth } from '@green-ecolution/backend-client'
import type { AlertProps } from '@green-ecolution/ui'
import type { StatusColor } from './types'

interface DataQualityFacts {
  dataHealth: DataHealth
  implausibleRecent: number
}

/// A sensor can have flagged values without being under defect suspicion, so
/// the display state is not the derived health alone.
export type DataQualityLevel = 'ok' | 'flagged' | 'suspect'

const DataQualityProperties: Record<
  DataQualityLevel,
  {
    color: StatusColor
    alert: NonNullable<AlertProps['variant']>
    label: string
    description: string
  }
> = {
  ok: {
    color: 'outline-green-dark',
    alert: 'success',
    label: 'Daten plausibel',
    description: 'Die zuletzt empfangenen Messwerte liegen im erwarteten Bereich.',
  },
  flagged: {
    color: 'outline-yellow',
    alert: 'warning',
    label: 'Einzelne Werte verworfen',
    description:
      'Einzelne Messwerte waren unplausibel und wurden von der Auswertung ausgeschlossen. Der Bewässerungszustand beruht auf den übrigen Werten.',
  },
  suspect: {
    color: 'outline-red',
    alert: 'destructive',
    label: 'Datenqualität prüfen',
    description:
      'Mehrere Übertragungen in Folge enthielten keinen verwertbaren Messwert. Der Sensor ist womöglich defekt und der angezeigte Bewässerungszustand veraltet.',
  },
}

export const dataQualityLevel = (sensor: DataQualityFacts): DataQualityLevel => {
  if (sensor.dataHealth === DataHealth.Suspect) return 'suspect'
  return sensor.implausibleRecent > 0 ? 'flagged' : 'ok'
}

export const getDataQualityDetails = (sensor: DataQualityFacts) =>
  DataQualityProperties[dataQualityLevel(sensor)]

export const hasQualityWarning = (sensor: DataQualityFacts): boolean =>
  dataQualityLevel(sensor) !== 'ok'

const QUALITY_REASON_LABEL: Record<string, string> = {
  out_of_range: 'Wert außerhalb des möglichen Bereichs',
  implausible_jump: 'Unplausibler Sprung gegenüber dem Vorwert',
}

export const qualityReasonLabel = (reason: string): string =>
  QUALITY_REASON_LABEL[reason] ?? 'Unbekannter Grund'
