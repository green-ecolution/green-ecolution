import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'
import type { EnumsTranslate, StatusColor } from './types'

const wateringPlanStatusColors: Record<WateringPlanStatus, StatusColor> = {
  [WateringPlanStatus.Active]: 'outline-green-light',
  [WateringPlanStatus.Canceled]: 'outline-red',
  [WateringPlanStatus.Finished]: 'outline-green-dark',
  [WateringPlanStatus.NotCompleted]: 'outline-dark',
  [WateringPlanStatus.Planned]: 'outline-dark',
}

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
    (status: WateringPlanStatus): WateringPlanStatusDetails => ({
      value: status,
      color: wateringPlanStatusColors[status],
      label: translate(`wateringPlanStatus.${status}.label`),
      description: translate(`wateringPlanStatus.${status}.description`),
    }),
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
