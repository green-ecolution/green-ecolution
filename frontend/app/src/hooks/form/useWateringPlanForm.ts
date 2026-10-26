import { WATERING_PLAN_FORM } from '@/store/form/formDraftSlice'
import { queryRoots } from '@/api/queries'
import type { PlanResponse } from '@/api/backendApi'
import { planApi } from '@/api/backendApi'
import { toPlanCreateRequest, toPlanFormUpdateRequest } from '@/lib/plans'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { wateringPlanDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

export const useWateringPlanForm = (
  mutationType: 'create' | 'update',
  opts: { wateringPlanId?: string; initForm?: DefaultValues<WateringPlanForm> },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('wateringPlan')

  const wateringPlanConfig: EntityFormConfig<
    WateringPlanForm,
    WateringPlanForm,
    WateringPlanForm,
    PlanResponse
  > = {
    formType: WATERING_PLAN_FORM,
    resolver: wateringPlanDraftResolver<WateringPlanForm>(translate),

    createFn: (body) => planApi.createPlan({ planCreateRequest: toPlanCreateRequest(body) }),
    updateFn: (id, body) =>
      planApi.updatePlan({ planId: id, planUpdateRequest: toPlanFormUpdateRequest(body) }),

    invalidates: queryRoots.wateringPlan,

    successRoute: (id) => ({
      to: '/watering-plans/$wateringPlanId',
      params: { wateringPlanId: id.toString() },
    }),
    replaceOnSuccess: true,
    allowedPaths: ['/map/watering-plan/select/cluster'],

    messages: {
      createLeave: t('form.createLeaveMessage'),
      updateLeave: t('form.updateLeaveMessage'),
      createSuccess: t('form.createSuccessMessage'),
      updateSuccess: t('form.updateSuccessMessage'),
    },
  }

  return useEntityForm<WateringPlanForm, WateringPlanForm, WateringPlanForm, PlanResponse>(
    wateringPlanConfig,
    mutationType,
    {
      entityId: opts.wateringPlanId,
      initForm: opts.initForm,
    },
  )
}
