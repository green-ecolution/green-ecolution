import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { PlanStatus } from '@green-ecolution/backend-client'
import type { PlanInListResponse } from '@green-ecolution/backend-client'
import type { EnumsTranslate, StatusColor } from './types'

const planStatusColors: Record<PlanStatus, StatusColor> = {
  [PlanStatus.Active]: 'outline-green-light',
  [PlanStatus.Canceled]: 'outline-red',
  [PlanStatus.Finished]: 'outline-green-dark',
  [PlanStatus.NotCompleted]: 'outline-dark',
  [PlanStatus.Planned]: 'outline-dark',
}

export interface PlanStatusDetails {
  value: PlanStatus
  color: StatusColor
  label: string
  description: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const usePlanStatusDetails = (): ((status: PlanStatus) => PlanStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: PlanStatus): PlanStatusDetails => ({
      value: status,
      color: planStatusColors[status],
      label: translate(`planStatus.${status}.label`),
      description: translate(`planStatus.${status}.description`),
    }),
    [translate],
  )
}

/**
 * Mirrors the transition table the backend enforces for operation plans.
 * Keep in sync — an entry missing here is unreachable in the UI, an extra one
 * only fails after submit with `InvalidStateTransition`.
 */
const planStatusTransitions: Record<PlanStatus, PlanStatus[]> = {
  [PlanStatus.Planned]: [PlanStatus.Active, PlanStatus.Canceled],
  [PlanStatus.Active]: [
    PlanStatus.Planned,
    PlanStatus.Finished,
    PlanStatus.NotCompleted,
    PlanStatus.Canceled,
  ],
  [PlanStatus.Finished]: [],
  [PlanStatus.Canceled]: [],
  [PlanStatus.NotCompleted]: [],
}

/**
 * Selectable statuses for a plan in `current`: the unchanged status first, then
 * its valid targets. Empty for terminal statuses, where the plan cannot move on.
 */
export const usePlanStatusTransitionOptions = (current: PlanStatus): PlanStatusDetails[] => {
  const getDetails = usePlanStatusDetails()
  return useMemo(() => {
    const targets = planStatusTransitions[current] ?? []
    if (targets.length === 0) return []
    return [current, ...targets].map(getDetails)
  }, [current, getDetails])
}

export const showPlanStatusButton = (plan: Pick<PlanInListResponse, 'status'>): boolean => {
  return (
    plan.status !== PlanStatus.NotCompleted &&
    plan.status !== PlanStatus.Finished &&
    plan.status !== PlanStatus.Canceled
  )
}
