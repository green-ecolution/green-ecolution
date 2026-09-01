import type { TFunction } from 'i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'

export type BoardColumnId = 'planned' | 'active' | 'done'

export const DONE_STATUSES: WateringPlanStatus[] = [
  WateringPlanStatus.Finished,
  WateringPlanStatus.Canceled,
  WateringPlanStatus.NotCompleted,
]

export function columnForStatus(status: WateringPlanStatus): BoardColumnId | null {
  switch (status) {
    case WateringPlanStatus.Planned:
      return 'planned'
    case WateringPlanStatus.Active:
      return 'active'
    case WateringPlanStatus.Finished:
    case WateringPlanStatus.Canceled:
    case WateringPlanStatus.NotCompleted:
      return 'done'
    case WateringPlanStatus.Unknown:
      return null
  }
}

export type DropAction = 'start' | 'cancel' | 'complete'

export function dropActionFor(from: BoardColumnId, to: BoardColumnId): DropAction | null {
  if (from === 'planned' && to === 'active') return 'start'
  if (from === 'planned' && to === 'done') return 'cancel'
  if (from === 'active' && to === 'done') return 'complete'
  return null
}

// Not a component: the caller passes its own scoped `t` (see clusterStatusReason.ts
// for the same shape) rather than this module reaching for `getI18n()` — a frozen
// translation here would reproduce the bug the router helpers had.
export function dropHintFor(action: DropAction, t: TFunction<'wateringPlan'>): string {
  switch (action) {
    case 'start':
      return t('board.dropHint.start')
    case 'cancel':
      return t('board.dropHint.cancel')
    case 'complete':
      return t('board.dropHint.complete')
  }
}
