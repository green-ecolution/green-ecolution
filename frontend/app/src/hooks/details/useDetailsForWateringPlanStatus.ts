import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import type { BadgeProps } from '@green-ecolution/ui'

export type StatusColor = NonNullable<BadgeProps['variant']>

export const WateringPlanStatusOptions: {
  value: WateringPlanStatus
  label: string
  color: StatusColor
  description: string
}[] = [
  {
    value: WateringPlanStatus.Unknown,
    label: 'Unbekannt',
    color: 'outline-dark',
    description: 'Der Status der Einsatzplanung ist unbekannt.',
  },
  {
    value: WateringPlanStatus.Active,
    label: 'Aktiv',
    color: 'outline-green-light',
    description: 'Der Einsatzplan ist aktiv und wird aktuell ausgeführt.',
  },
  {
    value: WateringPlanStatus.Canceled,
    label: 'Abgebrochen',
    color: 'outline-red',
    description: 'Der Einsatzplan wurde abgebrochen und ist nicht fertig gestellt.',
  },
  {
    value: WateringPlanStatus.Finished,
    label: 'Beendet',
    color: 'outline-green-dark',
    description: 'Der Einsatzplan wurde erfolgreich beendet.',
  },
  {
    value: WateringPlanStatus.NotCompeted,
    label: 'Nicht angetreten',
    color: 'outline-dark',
    description: 'Der Einsatzplan wurde nicht angetreten.',
  },
  {
    value: WateringPlanStatus.Planned,
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
    wateringPlan.status !== WateringPlanStatus.NotCompeted &&
    wateringPlan.status !== WateringPlanStatus.Finished &&
    wateringPlan.status !== WateringPlanStatus.Canceled
  )
}
