import { WateringPlanStatus } from '@green-ecolution/backend-client'

export type BoardColumnId = 'planned' | 'active' | 'done'

export const DONE_STATUSES: WateringPlanStatus[] = [
  WateringPlanStatus.Finished,
  WateringPlanStatus.Canceled,
  WateringPlanStatus.NotCompeted,
]

export function columnForStatus(status: WateringPlanStatus): BoardColumnId | null {
  switch (status) {
    case WateringPlanStatus.Planned:
      return 'planned'
    case WateringPlanStatus.Active:
      return 'active'
    case WateringPlanStatus.Finished:
    case WateringPlanStatus.Canceled:
    case WateringPlanStatus.NotCompeted:
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

export function dropHintFor(action: DropAction): string {
  switch (action) {
    case 'start':
      return 'Einsatz starten'
    case 'cancel':
      return 'Einsatz abbrechen'
    case 'complete':
      return 'Einsatz abschließen'
  }
}
