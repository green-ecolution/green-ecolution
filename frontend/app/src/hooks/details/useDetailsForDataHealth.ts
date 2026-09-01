import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { DataHealth } from '@green-ecolution/backend-client'
import type { AlertProps } from '@green-ecolution/ui'
import type { StatusColor } from './types'

// `t`'s generated overloads only accept the catalog's literal key union; the
// dynamic level/reason plugged into the template isn't statically one of those literals.
type EnumsTranslate = (key: string) => string

interface DataQualityFacts {
  dataHealth: DataHealth
  implausibleRecent: number
}

/// A sensor can have flagged values without being under defect suspicion, so
/// the display state is not the derived health alone.
export type DataQualityLevel = 'ok' | 'flagged' | 'suspect'

const DataQualityColors: Record<
  DataQualityLevel,
  { color: StatusColor; alert: NonNullable<AlertProps['variant']> }
> = {
  ok: { color: 'outline-green-dark', alert: 'success' },
  flagged: { color: 'outline-yellow', alert: 'warning' },
  suspect: { color: 'outline-red', alert: 'destructive' },
}

export interface DataQualityDetails {
  color: StatusColor
  alert: NonNullable<AlertProps['variant']>
  label: string
  description: string
}

export const dataQualityLevel = (sensor: DataQualityFacts): DataQualityLevel => {
  if (sensor.dataHealth === DataHealth.Suspect) return 'suspect'
  return sensor.implausibleRecent > 0 ? 'flagged' : 'ok'
}

export const hasQualityWarning = (sensor: DataQualityFacts): boolean =>
  dataQualityLevel(sensor) !== 'ok'

/** Reactive to language change: re-renders whichever component calls it. */
export const useDataQualityDetails = (): ((sensor: DataQualityFacts) => DataQualityDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (sensor: DataQualityFacts): DataQualityDetails => {
      const level = dataQualityLevel(sensor)
      return {
        ...DataQualityColors[level],
        label: translate(`dataHealth.${level}.label`),
        description: translate(`dataHealth.${level}.description`),
      }
    },
    [translate],
  )
}

const QUALITY_REASON_KEYS = new Set(['out_of_range', 'implausible_jump'])

/** Reactive to language change: re-renders whichever component calls it. */
export const useQualityReasonLabel = (): ((reason: string) => string) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (reason: string) =>
      translate(`dataHealth.reasons.${QUALITY_REASON_KEYS.has(reason) ? reason : 'unknown'}`),
    [translate],
  )
}
