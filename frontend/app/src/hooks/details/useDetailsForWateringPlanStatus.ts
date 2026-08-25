import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import { createEnumLookup } from '@/lib/enumLookup'
import type { StatusColor } from './types'

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

export const getWateringPlanStatusDetails = createEnumLookup(WateringPlanStatusOptions)

/**
 * Mirrors the transition table the backend enforces in `update_watering_plan`.
 * Keep in sync — an entry missing here is unreachable in the UI, an extra one
 * only fails after submit with `InvalidStateTransition`.
 */
const wateringPlanStatusTransitions: Record<WateringPlanStatus, WateringPlanStatus[]> = {
  [WateringPlanStatus.Planned]: [WateringPlanStatus.Active, WateringPlanStatus.Canceled],
  [WateringPlanStatus.Active]: [
    WateringPlanStatus.Planned,
    WateringPlanStatus.Finished,
    WateringPlanStatus.NotCompeted,
    WateringPlanStatus.Canceled,
  ],
  [WateringPlanStatus.Finished]: [],
  [WateringPlanStatus.Canceled]: [],
  [WateringPlanStatus.NotCompeted]: [],
  [WateringPlanStatus.Unknown]: [],
}

/**
 * Selectable statuses for a plan in `current`: the unchanged status first, then
 * its valid targets. Empty for terminal statuses, where the plan cannot move on.
 */
export const getWateringPlanStatusTransitionOptions = (current: WateringPlanStatus) => {
  const targets = wateringPlanStatusTransitions[current] ?? []
  if (targets.length === 0) return []
  return [current, ...targets].map(getWateringPlanStatusDetails)
}

export const showWateringPlanStatusButton = (wateringPlan: WateringPlan): boolean => {
  return (
    wateringPlan.status !== WateringPlanStatus.NotCompeted &&
    wateringPlan.status !== WateringPlanStatus.Finished &&
    wateringPlan.status !== WateringPlanStatus.Canceled
  )
}
