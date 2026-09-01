import { useMutation } from '@tanstack/react-query'
import createToast from '@/hooks/createToast'
import { useNavigate } from '@tanstack/react-router'
import type { Aggregate } from '@/api/queries'
import { useInvalidateAggregates } from '@/lib/queryInvalidation'
import { DefaultValues, FieldValues, Resolver, useForm } from 'react-hook-form'
import { useFormNavigationBlocker } from './useFormNavigationBlocker'
import { useFormDraft } from '@/store/form/useFormDraft'
import { useCallback } from 'react'
import { FormType, MutationType } from '@/store/form/formDraftSlice'
import { toApiError } from '@/lib/apiError'
import { useTranslation } from 'react-i18next'

export interface EntityFormConfig<TForm extends FieldValues, TCreate, TUpdate, TEntity> {
  formType: FormType
  resolver: Resolver<TForm>

  createFn: (body: TCreate) => Promise<TEntity>
  updateFn: (id: string, body: TUpdate) => Promise<TEntity>

  /** Every aggregate a save touches, including neighbours it changes indirectly. */
  invalidates: readonly Aggregate[]

  successRoute: (id: string) => { to: string; params: Record<string, string> }
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
  disableNavigationBlock?: boolean
}

export function useEntityForm<
  TForm extends FieldValues,
  TCreate,
  TUpdate,
  TEntity extends { id: string },
>(
  config: EntityFormConfig<TForm, TCreate, TUpdate, TEntity>,
  mutationType: MutationType,
  opts: EntityFormOptions<TForm>,
) {
  const showToast = createToast()
  const invalidate = useInvalidateAggregates()
  const navigate = useNavigate()
  const { t } = useTranslation('errors')
  const draft = useFormDraft<TForm>(config.formType, mutationType)

  const form = useForm<TForm>({
    defaultValues: opts.initForm,
    resolver: config.resolver,
  })

  const saveDraft = useCallback(() => {
    const data = form.getValues()
    if (data && Object.keys(data).length > 0) {
      draft.setData(data)
    }
  }, [form, draft])

  const navigationBlocker = useFormNavigationBlocker({
    isDirty: opts.disableNavigationBlock ? false : form.formState.isDirty || draft.hasChanges,
    allowedPaths: config.allowedPaths,
    onLeave: draft.clear,
    message: mutationType === 'create' ? config.messages.createLeave : config.messages.updateLeave,
  })

  const { mutate, isError, error } = useMutation({
    mutationFn: async (data: TCreate | TUpdate) => {
      try {
        if (mutationType === 'create') {
          return await config.createFn(data as TCreate)
        } else if (mutationType === 'update' && opts.entityId) {
          return await config.updateFn(opts.entityId, data as TUpdate)
        }
        throw new Error(`Invalid mutation type or missing entityId for update`)
      } catch (error) {
        throw await toApiError(error)
      }
    },

    onSuccess: (data: TEntity) => {
      draft.clear()
      // Not awaited on purpose: invalidateQueries marks the entries stale
      // synchronously, so the loader of the route we navigate to refetches.
      invalidate(config.invalidates).catch((error) =>
        console.error(`Invalidation after ${config.formType} mutation failed:`, error),
      )

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
      // The mutationFn's catch above already routed this through toApiError, so
      // `message` is the resolved catalog text, not the client's raw error.
      const { message } = error
      showToast(t('frame.formSubmitFailed', { reason: message }), 'error')
    },
    // A rejected save must not tear down the page: the form keeps the user's
    // input and renders the message inline instead.
    throwOnError: false,
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
