import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { SensorStatus } from '@green-ecolution/backend-client'
import type { EnumsTranslate, StatusColor } from './types'

const SensorStatusColors: Record<SensorStatus, { color: StatusColor; colorHex: string }> = {
  [SensorStatus.Prepared]: { color: 'outline-dark', colorHex: '#A2A2A2' },
  [SensorStatus.Offline]: { color: 'outline-red', colorHex: '#E44E4D' },
  [SensorStatus.Online]: { color: 'outline-green-dark', colorHex: '#4C7741' },
}

export interface SensorStatusDetails {
  color: StatusColor
  colorHex: string
  label: string
  description: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useSensorStatusDetails = (): ((status: SensorStatus) => SensorStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: SensorStatus): SensorStatusDetails => ({
      ...SensorStatusColors[status],
      label: translate(`sensorStatus.${status}.label`),
      description: translate(`sensorStatus.${status}.description`),
    }),
    [translate],
  )
}
