import { WateringStatus } from '@green-ecolution/backend-client'
import { StatusColor } from './useDetailsForWateringPlanStatus'

const WateringStatusProperties: Record<
  WateringStatus,
  {
    color: StatusColor
    label: string
    description: string
    colorHex: string
  }
> = {
  [WateringStatus.WateringStatusUnknown]: {
    color: 'dark-400',
    label: 'Unbekannt',
    description: 'Der Bewässerungsstatus ist unbekannt.',
    colorHex: '#A2A2A2',
  },
  [WateringStatus.WateringStatusJustWatered]: {
    color: 'dark-600',
    label: 'Soeben bewässert',
    description: 'Die Bäume wurden vor kurzem bewässert.',
    colorHex: '#747474',
  },
  [WateringStatus.WateringStatusBad]: {
    color: 'red',
    label: 'Sehr trocken',
    description: 'Die Bäume benötigen dringend Wasser.',
    colorHex: '#E44E4D',
  },
  [WateringStatus.WateringStatusModerate]: {
    color: 'yellow',
    label: 'Leicht trocken',
    description: 'Die Bäume sind leicht trocken und benötigen etwas Wasser.',
    colorHex: '#FFC434',
  },
  [WateringStatus.WateringStatusGood]: {
    color: 'green-light',
    label: 'In Ordnung',
    description: 'Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich.',
    colorHex: '#ACB63B',
  },
} as const

type WateringStatusDetails = (typeof WateringStatusProperties)[WateringStatus]

export const getWateringStatusDetails = (status: WateringStatus): WateringStatusDetails => {
  return WateringStatusProperties[status]
}
