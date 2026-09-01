import { useMutation, useQueryClient } from '@tanstack/react-query'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type {
  ListResponseWateringPlanInListResponse,
  WateringPlanUpdateRequest,
} from '@green-ecolution/backend-client'
import { useTranslation } from 'react-i18next'

export type PlanEvaluation = NonNullable<WateringPlanUpdateRequest['evaluation']>

import { wateringPlanApi } from '@/api/backendApi'
import type { WateringPlanInList } from '@/api/backendApi'
import { wateringPlanQueries, type Aggregate } from '@/api/queries'
import { PLAN_STATUS_AGGREGATES, useInvalidateAggregates } from '@/lib/queryInvalidation'
import createToast from '@/hooks/createToast'
import { resolveApiError } from '@/lib/apiError'

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
  provider: plan.provider,
  additionalInformation: plan.additionalInformation,
  ...overrides,
})

const plannedKey = wateringPlanQueries.boardColumn([WateringPlanStatus.Planned]).queryKey
const activeKey = wateringPlanQueries.boardColumn([WateringPlanStatus.Active]).queryKey

export const useWateringPlanBoardMutations = () => {
  const queryClient = useQueryClient()
  const invalidate = useInvalidateAggregates()
  const showToast = createToast()
  const { t } = useTranslation('errors')

  // The board stays mounted after a mutation, so the route loaders have to
  // re-run as well; cancelQueries elsewhere is scoped to the board to avoid
  // aborting unrelated watering-plan queries.
  const runInvalidation = (aggregates: readonly Aggregate[]) =>
    invalidate(aggregates, { reloadRoutes: true }).catch((error) =>
      console.error('Invalidation after watering plan mutation failed:', error),
    )

  const invalidateAfterStatusChange = () => void runInvalidation(PLAN_STATUS_AGGREGATES)
  const invalidatePlans = () => void runInvalidation(['wateringPlan'])

  const update = (plan: WateringPlanInList, overrides: Partial<WateringPlanUpdateRequest>) =>
    wateringPlanApi.updateWateringPlan({
      wateringPlanId: plan.id.toString(),
      wateringPlanUpdateRequest: toUpdateRequest(plan, overrides),
    })

  const revertStart = useMutation({
    mutationFn: (plan: WateringPlanInList) => update(plan, { status: WateringPlanStatus.Planned }),
    onMutate: async (plan) => {
      await queryClient.cancelQueries({ queryKey: ['watering-plans', 'board'] })
      const previousPlanned =
        queryClient.getQueryData<ListResponseWateringPlanInListResponse>(plannedKey)
      const previousActive =
        queryClient.getQueryData<ListResponseWateringPlanInListResponse>(activeKey)
      if (previousActive) {
        queryClient.setQueryData(activeKey, {
          ...previousActive,
          data: previousActive.data.filter((p) => p.id !== plan.id),
        })
      }
      if (previousPlanned) {
        queryClient.setQueryData(plannedKey, {
          ...previousPlanned,
          data: [{ ...plan, status: WateringPlanStatus.Planned }, ...previousPlanned.data],
        })
      }
      return { previousPlanned, previousActive }
    },
    onError: (error, _plan, context) => {
      if (context?.previousPlanned) queryClient.setQueryData(plannedKey, context.previousPlanned)
      if (context?.previousActive) queryClient.setQueryData(activeKey, context.previousActive)
      void resolveApiError(error).then((info) =>
        showToast(t('frame.wateringPlanStartRevertFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => showToast('Start rückgängig gemacht.'),
    onSettled: invalidateAfterStatusChange,
  })

  const startPlan = useMutation({
    mutationFn: (plan: WateringPlanInList) => update(plan, { status: WateringPlanStatus.Active }),
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
      void resolveApiError(error).then((info) =>
        showToast(t('frame.wateringPlanStartFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: (_data, plan) =>
      showToast('Einsatz gestartet.', 'success', {
        action: { label: 'Rückgängig', onClick: () => revertStart.mutate(plan) },
      }),
    onSettled: invalidateAfterStatusChange,
  })

  const cancelPlan = useMutation({
    mutationFn: ({ plan, note }: { plan: WateringPlanInList; note: string }) =>
      update(plan, { status: WateringPlanStatus.Canceled, cancellationNote: note }),
    onError: (error) => {
      void resolveApiError(error).then((info) =>
        showToast(t('frame.wateringPlanCancelFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => {
      showToast('Einsatz abgebrochen.')
      invalidateAfterStatusChange()
    },
  })

  const finishPlan = useMutation({
    mutationFn: ({ plan, evaluation }: { plan: WateringPlanInList; evaluation: PlanEvaluation }) =>
      update(plan, { status: WateringPlanStatus.Finished, evaluation }),
    onError: (error) => {
      void resolveApiError(error).then((info) =>
        showToast(t('frame.wateringPlanFinishFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => {
      showToast('Einsatz abgeschlossen.')
      invalidateAfterStatusChange()
    },
  })

  const assignUsers = useMutation({
    mutationFn: ({ plan, userIds }: { plan: WateringPlanInList; userIds: string[] }) =>
      update(plan, { userIds }),
    onError: (error) => {
      void resolveApiError(error).then((info) =>
        showToast(t('frame.wateringPlanAssignFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => {
      showToast('Zuweisung gespeichert.')
      invalidatePlans()
    },
  })

  return { revertStart, startPlan, cancelPlan, finishPlan, assignUsers }
}
