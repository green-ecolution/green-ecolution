import { queryRoots } from '@/api/queries'
import { useMutation, useQueryClient, type QueryKey } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import type {
  ListResponsePlanInListResponse,
  PlanInListResponse,
  PlanUpdateRequest,
  StopOutcomeDto,
} from '@green-ecolution/backend-client'
import { planApi } from '@/api/backendApi'
import { wateringPlanQueries } from '@/api/queries'
import { planStatusQueryRoots, useInvalidateQueryRoots } from '@/lib/queryInvalidation'
import { toUpdateRequest } from '@/lib/plans'
import { completeOutcomes } from '@/modules/planning'
import type { ModulePlanning } from '@/modules/types'
import createToast from '@/hooks/createToast'
import { resolveApiError } from '@/lib/apiError'

const columnKeys = (plan: PlanInListResponse) => ({
  plannedKey: wateringPlanQueries.boardColumn(plan.module, [WateringPlanStatus.Planned]).queryKey,
  activeKey: wateringPlanQueries.boardColumn(plan.module, [WateringPlanStatus.Active]).queryKey,
})

/** `planning` names what a status change invalidates beyond the plans; assignments need none. */
export const useWateringPlanBoardMutations = (
  planning?: Pick<ModulePlanning, 'invalidateOnStatusChange'>,
) => {
  const queryClient = useQueryClient()
  const invalidate = useInvalidateQueryRoots()
  const showToast = createToast()
  // Default namespace stays 'errors' so every existing `t('frame.*')` call below
  // is untouched; new wateringPlan strings are reached with an explicit prefix.
  const { t } = useTranslation(['errors', 'wateringPlan'])

  // The board stays mounted after a mutation, so the route loaders have to
  // re-run as well; cancelQueries elsewhere is scoped to the board to avoid
  // aborting unrelated watering-plan queries.
  const runInvalidation = (roots: readonly QueryKey[]) =>
    invalidate(roots, { reloadRoutes: true }).catch((error) =>
      console.error('Invalidation after watering plan mutation failed:', error),
    )

  const invalidateAfterStatusChange = () => void runInvalidation(planStatusQueryRoots(planning))
  const invalidatePlans = () => void runInvalidation(queryRoots.wateringPlan)

  const update = (plan: PlanInListResponse, overrides: Partial<PlanUpdateRequest>) =>
    planApi.updatePlan({ planId: plan.id, planUpdateRequest: toUpdateRequest(plan, overrides) })

  const revertStart = useMutation({
    mutationFn: (plan: PlanInListResponse) => update(plan, { status: WateringPlanStatus.Planned }),
    onMutate: async (plan) => {
      await queryClient.cancelQueries({ queryKey: ['watering-plans', 'board'] })
      const { plannedKey, activeKey } = columnKeys(plan)
      const previousPlanned = queryClient.getQueryData<ListResponsePlanInListResponse>(plannedKey)
      const previousActive = queryClient.getQueryData<ListResponsePlanInListResponse>(activeKey)
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
    onError: (error, plan, context) => {
      const { plannedKey, activeKey } = columnKeys(plan)
      if (context?.previousPlanned) queryClient.setQueryData(plannedKey, context.previousPlanned)
      if (context?.previousActive) queryClient.setQueryData(activeKey, context.previousActive)
      void resolveApiError(error, { module: plan.module }).then((info) =>
        showToast(t('frame.wateringPlanStartRevertFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => showToast(t('wateringPlan:board.mutation.startRevertedToast')),
    onSettled: invalidateAfterStatusChange,
  })

  const startPlan = useMutation({
    mutationFn: (plan: PlanInListResponse) => update(plan, { status: WateringPlanStatus.Active }),
    onMutate: async (plan) => {
      await queryClient.cancelQueries({ queryKey: ['watering-plans', 'board'] })
      const { plannedKey, activeKey } = columnKeys(plan)
      const previousPlanned = queryClient.getQueryData<ListResponsePlanInListResponse>(plannedKey)
      const previousActive = queryClient.getQueryData<ListResponsePlanInListResponse>(activeKey)
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
    onError: (error, plan, context) => {
      const { plannedKey, activeKey } = columnKeys(plan)
      if (context?.previousPlanned) queryClient.setQueryData(plannedKey, context.previousPlanned)
      if (context?.previousActive) queryClient.setQueryData(activeKey, context.previousActive)
      void resolveApiError(error, { module: plan.module }).then((info) =>
        showToast(t('frame.wateringPlanStartFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: (_data, plan) =>
      showToast(t('wateringPlan:board.mutation.startedToast'), 'success', {
        action: {
          label: t('wateringPlan:board.mutation.undoActionLabel'),
          onClick: () => revertStart.mutate(plan),
        },
      }),
    onSettled: invalidateAfterStatusChange,
  })

  const cancelPlan = useMutation({
    mutationFn: ({ plan, note }: { plan: PlanInListResponse; note: string }) =>
      update(plan, { status: WateringPlanStatus.Canceled, cancellationNote: note }),
    onError: (error, { plan }) => {
      void resolveApiError(error, { module: plan.module }).then((info) =>
        showToast(t('frame.wateringPlanCancelFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => {
      showToast(t('wateringPlan:board.mutation.canceledToast'))
      invalidateAfterStatusChange()
    },
  })

  const finishPlan = useMutation({
    mutationFn: ({ plan, outcomes }: { plan: PlanInListResponse; outcomes: StopOutcomeDto[] }) =>
      update(plan, {
        status: WateringPlanStatus.Finished,
        outcomes: completeOutcomes(plan.stops, outcomes),
      }),
    onError: (error, { plan }) => {
      void resolveApiError(error, { module: plan.module }).then((info) =>
        showToast(t('frame.wateringPlanFinishFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => {
      showToast(t('wateringPlan:board.mutation.finishedToast'))
      invalidateAfterStatusChange()
    },
  })

  const assignUsers = useMutation({
    mutationFn: ({ plan, userIds }: { plan: PlanInListResponse; userIds: string[] }) =>
      update(plan, { userIds }),
    onError: (error, { plan }) => {
      void resolveApiError(error, { module: plan.module }).then((info) =>
        showToast(t('frame.wateringPlanAssignFailed', { reason: info.message }), 'error'),
      )
    },
    onSuccess: () => {
      showToast(t('wateringPlan:board.mutation.assignedToast'))
      invalidatePlans()
    },
  })

  return { revertStart, startPlan, cancelPlan, finishPlan, assignUsers }
}
