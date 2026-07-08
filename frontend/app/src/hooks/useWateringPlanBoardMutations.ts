import { useMutation, useQueryClient } from '@tanstack/react-query'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type {
  ListResponseWateringPlanInListResponse,
  WateringPlanUpdateRequest,
} from '@green-ecolution/backend-client'

export type PlanEvaluation = NonNullable<WateringPlanUpdateRequest['evaluation']>

import { wateringPlanApi } from '@/api/backendApi'
import type { WateringPlanInList } from '@/api/backendApi'
import { wateringPlanBoardColumnQuery } from '@/api/queries'
import createToast from '@/hooks/createToast'

export const toUpdateRequest = (
  plan: WateringPlanInList,
  overrides: Partial<WateringPlanUpdateRequest>,
): WateringPlanUpdateRequest => ({
  date: plan.date,
  description: plan.description,
  status: plan.status,
  transporterId: plan.transporter.id,
  trailerId: plan.trailer?.id,
  treeClusterIds: plan.treeclusters.map((cluster) => cluster.id),
  userIds: plan.userIds,
  cancellationNote: plan.cancellationNote,
  ...overrides,
})

const plannedKey = wateringPlanBoardColumnQuery([WateringPlanStatus.Planned]).queryKey
const activeKey = wateringPlanBoardColumnQuery([WateringPlanStatus.Active]).queryKey

export const useWateringPlanBoardMutations = () => {
  const queryClient = useQueryClient()
  const showToast = createToast()

  const invalidateBoard = () => {
    queryClient
      .invalidateQueries({ queryKey: ['watering-plans'] })
      .catch((error) => console.error('Invalidate watering-plans failed:', error))
  }

  const update = (plan: WateringPlanInList, overrides: Partial<WateringPlanUpdateRequest>) =>
    wateringPlanApi.updateWateringPlan({
      wateringPlanId: plan.id.toString(),
      wateringPlanUpdateRequest: toUpdateRequest(plan, overrides),
    })

  const startPlan = useMutation({
    mutationFn: (plan: WateringPlanInList) =>
      update(plan, { status: WateringPlanStatus.Active }),
    onMutate: async (plan) => {
      await queryClient.cancelQueries({ queryKey: ['watering-plans', 'board'] })
      const previousPlanned =
        queryClient.getQueryData<ListResponseWateringPlanInListResponse>(plannedKey)
      const previousActive =
        queryClient.getQueryData<ListResponseWateringPlanInListResponse>(activeKey)
      if (previousPlanned) {
        queryClient.setQueryData(plannedKey, {
          ...previousPlanned,
          data: previousPlanned.data.filter((p) => p.id !== plan.id),
        })
      }
      if (previousActive) {
        queryClient.setQueryData(activeKey, {
          ...previousActive,
          data: [{ ...plan, status: WateringPlanStatus.Active }, ...previousActive.data],
        })
      }
      return { previousPlanned, previousActive }
    },
    onError: (error, _plan, context) => {
      if (context?.previousPlanned) queryClient.setQueryData(plannedKey, context.previousPlanned)
      if (context?.previousActive) queryClient.setQueryData(activeKey, context.previousActive)
      showToast(`Der Einsatz konnte nicht gestartet werden: ${error.message}`, 'error')
    },
    onSuccess: () => showToast('Einsatz gestartet.'),
    onSettled: invalidateBoard,
  })

  const cancelPlan = useMutation({
    mutationFn: ({ plan, note }: { plan: WateringPlanInList; note: string }) =>
      update(plan, { status: WateringPlanStatus.Canceled, cancellationNote: note }),
    onError: (error) =>
      showToast(`Der Einsatz konnte nicht abgebrochen werden: ${error.message}`, 'error'),
    onSuccess: () => {
      showToast('Einsatz abgebrochen.')
      invalidateBoard()
    },
  })

  const finishPlan = useMutation({
    mutationFn: ({ plan, evaluation }: { plan: WateringPlanInList; evaluation: PlanEvaluation }) =>
      update(plan, { status: WateringPlanStatus.Finished, evaluation }),
    onError: (error) =>
      showToast(`Der Einsatz konnte nicht abgeschlossen werden: ${error.message}`, 'error'),
    onSuccess: () => {
      showToast('Einsatz abgeschlossen.')
      invalidateBoard()
    },
  })

  const assignUsers = useMutation({
    mutationFn: ({ plan, userIds }: { plan: WateringPlanInList; userIds: string[] }) =>
      update(plan, { userIds }),
    onError: (error) =>
      showToast(`Die Zuweisung konnte nicht gespeichert werden: ${error.message}`, 'error'),
    onSuccess: () => {
      showToast('Zuweisung gespeichert.')
      invalidateBoard()
    },
  })

  return { startPlan, cancelPlan, finishPlan, assignUsers }
}
