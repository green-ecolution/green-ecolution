import { QueryClient } from '@tanstack/react-query'
import { wateringPlanIdQuery, wateringPlanQuery } from '@/api/queries'
import {
  WateringPlan,
  WateringPlanCreate,
  WateringPlanUpdate,
} from '@green-ecolution/backend-client'
import { wateringPlanApi } from '@/api/backendApi'
import { WateringPlanForm, wateringPlanSchema } from '@/schema/wateringPlanSchema'
import { DefaultValues } from 'react-hook-form'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

const wateringPlanConfig: EntityFormConfig<
  WateringPlanForm,
  WateringPlanCreate,
  WateringPlanUpdate,
  WateringPlan
> = {
  formType: 'wateringplan',
  schema: wateringPlanSchema,

  createFn: (body) => wateringPlanApi.createWateringPlan({ body }),
  updateFn: (id, body) => wateringPlanApi.updateWateringPlan({ id: Number(id), body }),

  invalidateQueries: (data, queryClient: QueryClient) => {
    queryClient
      .invalidateQueries(wateringPlanIdQuery(String(data.id)))
      .catch((error) => console.error('Invalidate "wateringPlanIdQuery" failed', error))
    queryClient
      .invalidateQueries(wateringPlanQuery())
      .catch((error) => console.error('Invalidate "wateringPlanQuery" failed:', error))
  },

  successRoute: (id) => ({
    to: '/watering-plans/$wateringPlanId',
    params: { wateringPlanId: id.toString() },
  }),
  replaceOnSuccess: true,
  allowedPaths: ['/map/watering-plan/select/cluster'],

  messages: {
    createLeave:
      'Möchtest du die Seite wirklich verlassen? Deine Eingaben zum Erstellen des Einsatzplans gehen verloren, wenn du jetzt gehst.',
    updateLeave:
      'Möchtest du die Seite wirklich verlassen? Deine Änderungen am Einsatzplan gehen verloren, wenn du jetzt gehst.',
    createSuccess: 'Der Einsatzplan wurde erfolgreich erstellt.',
    updateSuccess: 'Der Einsatzplan wurde erfolgreich bearbeitet.',
  },
}

export const useWateringPlanForm = (
  mutationType: 'create' | 'update',
  opts: { wateringPlanId?: string; initForm?: DefaultValues<WateringPlanForm> },
) => {
  return useEntityForm<WateringPlanForm, WateringPlanCreate, WateringPlanUpdate, WateringPlan>(
    wateringPlanConfig,
    mutationType,
    {
      entityId: opts.wateringPlanId,
      initForm: opts.initForm,
    },
  )
}
