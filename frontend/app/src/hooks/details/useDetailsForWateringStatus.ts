import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { WateringStatus } from '@green-ecolution/backend-client'
import { EnumsTranslate, StatusColor } from './types'

const WateringStatusColors: Record<WateringStatus, { color: StatusColor; colorHex: string }> = {
  [WateringStatus.Unknown]: { color: 'outline-dark', colorHex: '#A2A2A2' },
  [WateringStatus.JustWatered]: { color: 'outline-dark', colorHex: '#747474' },
  [WateringStatus.Bad]: { color: 'outline-red', colorHex: '#E44E4D' },
  [WateringStatus.Moderate]: { color: 'outline-yellow', colorHex: '#FFC434' },
  [WateringStatus.Good]: { color: 'outline-green-light', colorHex: '#ACB63B' },
}

export interface WateringStatusDetails {
  color: StatusColor
  colorHex: string
  label: string
  description: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useWateringStatusDetails = (): ((status: WateringStatus) => WateringStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: WateringStatus): WateringStatusDetails => ({
      ...WateringStatusColors[status],
      label: translate(`wateringStatus.${status}.label`),
      description: translate(`wateringStatus.${status}.description`),
    }),
    [translate],
  )
}
