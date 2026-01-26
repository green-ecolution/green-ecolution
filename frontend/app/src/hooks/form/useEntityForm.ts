import { useMutation, useQueryClient, QueryClient } from '@tanstack/react-query'
import createToast from '@/hooks/createToast'
import { useNavigate } from '@tanstack/react-router'
import { DefaultValues, FieldValues, useForm } from 'react-hook-form'
import { zodResolver } from '@/lib/zodResolver'
import { useFormNavigationBlocker } from './useFormNavigationBlocker'
import { useFormDraft } from '@/store/form/useFormDraft'
import { useCallback } from 'react'
import { FormType, MutationType } from '@/store/form/formDraftSlice'
import { z } from 'zod'

export interface EntityFormConfig<TForm extends FieldValues, TCreate, TUpdate, TEntity> {
  formType: FormType
  schema: z.ZodType<TForm>

  createFn: (body: TCreate) => Promise<TEntity>
  updateFn: (id: string, body: TUpdate) => Promise<TEntity>

  invalidateQueries: (data: TEntity, queryClient: QueryClient) => void

  successRoute: (id: number) => { to: string; params: Record<string, string> }
  replaceOnSuccess?: boolean
  allowedPaths: string[]

  messages: {
    createLeave: string
    updateLeave: string
    createSuccess: string
    updateSuccess: string
  }
}

export interface EntityFormOptions<TForm> {
  entityId?: string
  initForm?: DefaultValues<TForm>
}

export function useEntityForm<
  TForm extends FieldValues,
  TCreate,
  TUpdate,
  TEntity extends { id: number },
>(
  config: EntityFormConfig<TForm, TCreate, TUpdate, TEntity>,
  mutationType: MutationType,
  opts: EntityFormOptions<TForm>,
) {
  const showToast = createToast()
  const queryClient = useQueryClient()
  const navigate = useNavigate()
  const draft = useFormDraft<TForm>(config.formType, mutationType)

  const form = useForm<TForm>({
    defaultValues: opts.initForm,
    resolver: zodResolver(config.schema),
  })

  const saveDraft = useCallback(() => {
    const data = form.getValues()
    if (data && Object.keys(data).length > 0) {
      draft.setData(data)
    }
  }, [form, draft])

  const navigationBlocker = useFormNavigationBlocker({
    isDirty: form.formState.isDirty || draft.hasChanges,
    allowedPaths: config.allowedPaths,
    onLeave: draft.clear,
    message: mutationType === 'create' ? config.messages.createLeave : config.messages.updateLeave,
  })

  const { mutate, isError, error } = useMutation({
    mutationFn: (data: TCreate | TUpdate) => {
      if (mutationType === 'create') {
        return config.createFn(data as TCreate)
      } else if (mutationType === 'update' && opts.entityId) {
        return config.updateFn(opts.entityId, data as TUpdate)
      }
      return Promise.reject(new Error(`Invalid mutation type or missing entityId for update`))
    },

    onSuccess: (data: TEntity) => {
      draft.clear()
      config.invalidateQueries(data, queryClient)

      navigationBlocker.allowNavigation()
      const route = config.successRoute(data.id)
      navigate({
        to: route.to,
        params: route.params,
        replace: config.replaceOnSuccess,
      }).catch((error) => console.error('Navigation failed:', error))

      showToast(
        mutationType === 'create' ? config.messages.createSuccess : config.messages.updateSuccess,
      )
    },

    onError: (error) => {
      console.error(`Error with ${config.formType} mutation:`, error)
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error')
    },
    throwOnError: true,
  })

  return {
    mutate,
    isError,
    error,
    form,
    navigationBlocker,
    saveDraft,
  }
}
