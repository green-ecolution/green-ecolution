import { useMutation, useQueryClient } from '@tanstack/react-query'
import { wateringPlanIdQuery, wateringPlanQuery } from '@/api/queries'
import useToast from '@/hooks/useToast'
import { useNavigate } from '@tanstack/react-router'
import {
  WateringPlan,
  WateringPlanCreate,
  WateringPlanUpdate,
} from '@green-ecolution/backend-client'
import { wateringPlanApi } from '@/api/backendApi'
import { WateringPlanForm, wateringPlanSchema } from '@/schema/wateringPlanSchema'
import { DefaultValues, useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import useFormPersist from './usePersistForm'

type MutationOption = 'create' | 'update'
type MutationType<T> = T extends 'create'
  ? WateringPlanCreate
  : T extends 'update'
    ? WateringPlanUpdate
    : never

export const useWaterinPlanForm = <T extends MutationOption>(
  mutationType: T,
  opts: { wateringPlanId?: string; initForm?: DefaultValues<WateringPlanForm> },
) => {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const form = useForm<WateringPlanForm>({
    defaultValues: opts.initForm,
    resolver: zodResolver(wateringPlanSchema),
  })

  const { clear: resetPersist } = useFormPersist(`${mutationType}-wateringplan`, {
    watch: form.watch,
  })

  const { mutate, isError, error } = useMutation<WateringPlan, Error, MutationType<T>>({
    mutationFn: (wateringPlan: MutationType<T>) => {
      if (mutationType === 'create') {
        return wateringPlanApi.createWateringPlan({
          body: wateringPlan,
        })
      } else if (mutationType === 'update' && opts.wateringPlanId) {
        return wateringPlanApi.updateWateringPlan({
          id: opts.wateringPlanId,
          body: wateringPlan as WateringPlanUpdate,
        })
      }
      return Promise.reject(Error('Invalid mutation type or missing wateringPlanId for update'))
    },

    onSuccess: (data: WateringPlan) => {
      resetPersist()
      queryClient
        .invalidateQueries(wateringPlanIdQuery(String(data.id)))
        .catch((error) => console.error('Invalidate "waterinPlanIdQuery" failed', error))
      queryClient
        .invalidateQueries(wateringPlanQuery())
        .catch((error) => console.error('Invalidate "wateringPlanQuery" failed:', error))

      navigate({
        to: `/watering-plans/$wateringPlanId`,
        params: { wateringPlanId: data.id.toString() },
        search: { resetStore: false },
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))

      if (mutationType === 'create') showToast('Der Einsatzplan wurde erfolgreich erstellt.')
      else showToast('Der Einsatzplan wurde erfolgreich bearbeitet.')
    },

    onError: (error) => {
      console.error('Error with vehicle mutation:', error)
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error') // TODO: Parse API ResponseError
    },
    throwOnError: true,
  })

  return {
    mutate,
    isError,
    error,
    form,
  }
}
