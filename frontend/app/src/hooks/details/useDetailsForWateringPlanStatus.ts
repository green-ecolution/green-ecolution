import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import type { StatusColor } from './types'

// `t`'s generated overloads only accept the catalog's literal key union; the
// enum value plugged into the template isn't statically one of those literals.
type EnumsTranslate = (key: string) => string

const WateringPlanStatusEntries: { value: WateringPlanStatus; color: StatusColor }[] = [
  { value: WateringPlanStatus.Unknown, color: 'outline-dark' },
  { value: WateringPlanStatus.Active, color: 'outline-green-light' },
  { value: WateringPlanStatus.Canceled, color: 'outline-red' },
  { value: WateringPlanStatus.Finished, color: 'outline-green-dark' },
  { value: WateringPlanStatus.NotCompleted, color: 'outline-dark' },
  { value: WateringPlanStatus.Planned, color: 'outline-dark' },
]

export interface WateringPlanStatusDetails {
  value: WateringPlanStatus
  color: StatusColor
  label: string
  description: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useWateringPlanStatusDetails = (): ((
  status: WateringPlanStatus,
) => WateringPlanStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: WateringPlanStatus): WateringPlanStatusDetails => {
      const entry =
        WateringPlanStatusEntries.find((option) => option.value === status) ??
        WateringPlanStatusEntries[0]
      return {
        ...entry,
        label: translate(`wateringPlanStatus.${entry.value}.label`),
        description: translate(`wateringPlanStatus.${entry.value}.description`),
      }
    },
    [translate],
  )
}

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
    WateringPlanStatus.NotCompleted,
    WateringPlanStatus.Canceled,
  ],
  [WateringPlanStatus.Finished]: [],
  [WateringPlanStatus.Canceled]: [],
  [WateringPlanStatus.NotCompleted]: [],
  [WateringPlanStatus.Unknown]: [],
}

/**
 * Selectable statuses for a plan in `current`: the unchanged status first, then
 * its valid targets. Empty for terminal statuses, where the plan cannot move on.
 */
export const useWateringPlanStatusTransitionOptions = (
  current: WateringPlanStatus,
): WateringPlanStatusDetails[] => {
  const getDetails = useWateringPlanStatusDetails()
  return useMemo(() => {
    const targets = wateringPlanStatusTransitions[current] ?? []
    if (targets.length === 0) return []
    return [current, ...targets].map(getDetails)
  }, [current, getDetails])
}

export const showWateringPlanStatusButton = (wateringPlan: WateringPlan): boolean => {
  return (
    wateringPlan.status !== WateringPlanStatus.NotCompleted &&
    wateringPlan.status !== WateringPlanStatus.Finished &&
    wateringPlan.status !== WateringPlanStatus.Canceled
  )
}
