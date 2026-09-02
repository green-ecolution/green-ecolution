import type { WateringPlan, WateringPlanCreate, WateringPlanUpdate } from '@/api/backendApi'
import { wateringPlanApi } from '@/api/backendApi'
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
    WateringPlanCreate,
    WateringPlanUpdate,
    WateringPlan
  > = {
    formType: 'wateringplan',
    resolver: wateringPlanDraftResolver<WateringPlanForm>(translate),

    createFn: (body) => wateringPlanApi.createWateringPlan({ wateringPlanCreateRequest: body }),
    updateFn: (id, body) =>
      wateringPlanApi.updateWateringPlan({
        wateringPlanId: id,
        wateringPlanUpdateRequest: body,
      }),

    invalidates: ['wateringPlan'],

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

  return useEntityForm<WateringPlanForm, WateringPlanCreate, WateringPlanUpdate, WateringPlan>(
    wateringPlanConfig,
    mutationType,
    {
      entityId: opts.wateringPlanId,
      initForm: opts.initForm,
    },
  )
}
