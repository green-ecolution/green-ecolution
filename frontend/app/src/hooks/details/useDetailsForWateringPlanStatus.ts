import { WateringPlan, WateringPlanStatus } from '@green-ecolution/backend-client'
import type { BadgeProps } from '@green-ecolution/ui'

export type StatusColor = NonNullable<BadgeProps['variant']>

export const WateringPlanStatusOptions: {
  value: WateringPlanStatus
  label: string
  color: StatusColor
  description: string
}[] = [
  {
    value: WateringPlanStatus.WateringPlanStatusUnknown,
    label: 'Unbekannt',
    color: 'outline-dark',
    description: 'Der Status der Einsatzplanung ist unbekannt.',
  },
  {
    value: WateringPlanStatus.WateringPlanStatusActive,
    label: 'Aktiv',
    color: 'outline-green-light',
    description: 'Der Einsatzplan ist aktiv und wird aktuell ausgeführt.',
  },
  {
    value: WateringPlanStatus.WateringPlanStatusCanceled,
    label: 'Abgebrochen',
    color: 'outline-red',
    description: 'Der Einsatzplan wurde abgebrochen und ist nicht fertig gestellt.',
  },
  {
    value: WateringPlanStatus.WateringPlanStatusFinished,
    label: 'Beendet',
    color: 'outline-green-dark',
    description: 'Der Einsatzplan wurde erfolgreich beendet.',
  },
  {
    value: WateringPlanStatus.WateringPlanStatusNotCompeted,
    label: 'Nicht angetreten',
    color: 'outline-dark',
    description: 'Der Einsatzplan wurde nicht angetreten.',
  },
  {
    value: WateringPlanStatus.WateringPlanStatusPlanned,
    label: 'Geplant',
    color: 'outline-dark',
    description: 'Der Einsatzplan ist geplant und kann gestartet werden.',
  },
]

export const getWateringPlanStatusDetails = (status: WateringPlanStatus) =>
  WateringPlanStatusOptions.find((option) => option.value === status) ??
  WateringPlanStatusOptions[0]

export const showWateringPlanStatusButton = (wateringPlan: WateringPlan): boolean => {
  return (
    wateringPlan.status !== WateringPlanStatus.WateringPlanStatusNotCompeted &&
    wateringPlan.status !== WateringPlanStatus.WateringPlanStatusFinished &&
    wateringPlan.status !== WateringPlanStatus.WateringPlanStatusCanceled
  )
}
