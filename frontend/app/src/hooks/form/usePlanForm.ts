import { PLAN_FORM } from '@/store/form/formDraftSlice'
import { queryRoots } from '@/api/queries'
import type { PlanResponse } from '@/api/backendApi'
import { planApi } from '@/api/backendApi'
import { toPlanCreateRequest, toPlanFormUpdateRequest } from '@/lib/plans'
import { useIssueTranslator } from '@/lib/i18n/validation'
import type { PlanningManifest } from '@/modules/planning'
import { PlanForm } from '@/schema/planSchema'
import { planDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

export const usePlanForm = (
  mutationType: 'create' | 'update',
  opts: {
    wateringPlanId?: string
    initForm?: DefaultValues<PlanForm>
    planManifest?: PlanningManifest
  },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('plan')

  const planConfig: EntityFormConfig<PlanForm, PlanForm, PlanForm, PlanResponse> = {
    formType: PLAN_FORM,
    resolver: planDraftResolver<PlanForm>(translate),

    createFn: (body) => planApi.createPlan({ planCreateRequest: toPlanCreateRequest(body) }),
    updateFn: (id, body) =>
      planApi.updatePlan({ planId: id, planUpdateRequest: toPlanFormUpdateRequest(body) }),

    invalidates: queryRoots.plan,

    successRoute: (id) => ({
      to: '/plans/$wateringPlanId',
      params: { wateringPlanId: id.toString() },
    }),
    replaceOnSuccess: true,
    allowedPaths: [...(opts.planManifest?.planning.stopPickerPaths ?? [])],
    errorModule: opts.planManifest?.key,

    messages: {
      createLeave: t('form.createLeaveMessage'),
      updateLeave: t('form.updateLeaveMessage'),
      createSuccess: t('form.createSuccessMessage'),
      updateSuccess: t('form.updateSuccessMessage'),
    },
  }

  return useEntityForm<PlanForm, PlanForm, PlanForm, PlanResponse>(planConfig, mutationType, {
    entityId: opts.wateringPlanId,
    initForm: opts.initForm,
  })
}
