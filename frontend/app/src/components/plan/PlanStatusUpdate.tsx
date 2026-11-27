import { Suspense, useCallback, useState } from 'react'
import FormPageHeader from '../general/FormPageHeader'
import { PlanStatus } from '@green-ecolution/backend-client'
import type {
  PlanInListResponse,
  PlanResponse,
  PlanUpdateRequest,
  StopOutcomeDto,
} from '@green-ecolution/backend-client'
import { planQueries } from '@/api/queries'
import { format } from 'date-fns'
import { MoveRight } from 'lucide-react'
import FormError from '../general/form/FormError'
import {
  usePlanStatusDetails,
  usePlanStatusTransitionOptions,
} from '@/hooks/details/useDetailsForPlanStatus'
import {
  Badge,
  TextareaField,
  SelectField,
  Button,
  InlineAlert,
  Loading,
} from '@green-ecolution/ui'
import { PlanCancelForm, planCancelSchema } from '@/schema/planSchema'
import { zodResolver } from '@/lib/zodResolver'
import { SubmitHandler, useForm } from 'react-hook-form'
import { useMutation, useSuspenseQuery } from '@tanstack/react-query'
import { planStatusQueryRoots, useInvalidateQueryRoots } from '@/lib/queryInvalidation'
import { planApi } from '@/api/backendApi'
import { useNavigate } from '@tanstack/react-router'
import createToast from '@/hooks/createToast'
import { useTranslation } from 'react-i18next'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { toApiError } from '@/lib/apiError'
import { toUpdateRequest } from '@/lib/plans'
import { completeOutcomes, planningFor, planTextKey } from '@/modules/planning'
import type { ModulePlanning } from '@/modules/types'
import { useActiveModules } from '@/modules/useActiveModules'

interface PlanStatusUpdateProps {
  planId: string
}

const PlanStatusUpdate = ({ planId }: PlanStatusUpdateProps) => {
  const { data: loadedData } = useSuspenseQuery(planQueries.detail(planId))
  const { manifests } = useActiveModules()
  const planning = planningFor(manifests, loadedData.module)
  const navigate = useNavigate()
  const invalidate = useInvalidateQueryRoots()
  const showToast = createToast()
  const { t, i18n } = useTranslation(['plan', 'errors', 'common'])
  const dateLocale = useDateLocale()
  const getPlanStatusDetails = usePlanStatusDetails()
  const statusDetails = getPlanStatusDetails(loadedData.status)
  const statusOptions = usePlanStatusTransitionOptions(loadedData.status)
  const [selectedStatus, setSelectedStatus] = useState(statusDetails)

  const { mutate, isError, error } = useMutation({
    mutationFn: async (overrides: Partial<PlanUpdateRequest>) => {
      try {
        return await planApi.updatePlan({
          planId: planId,
          planUpdateRequest: toUpdateRequest(loadedData, overrides),
        })
      } catch (error) {
        throw await toApiError(error, { module: loadedData.module })
      }
    },

    onSuccess: (data: PlanResponse) => {
      invalidate(planStatusQueryRoots(planning)).catch((error) =>
        console.error('Invalidation after plan status update failed:', error),
      )

      navigate({
        to: `/plans/$planId`,
        params: { planId: data.id.toString() },
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))

      showToast(t('statusUpdate.toastSuccess'))
    },

    onError: (error) => {
      console.error('Error with vehicle mutation:', error)
      // mutationFn already routed this through toApiError, so error.message is
      // the resolved catalog text — the same value FormError renders below.
      showToast(t('errors:frame.planStatusUpdateFailed', { reason: error.message }), 'error')
    },
    throwOnError: true,
  })

  const date = format(new Date(loadedData.date), 'dd.MM.yyyy', { locale: dateLocale })

  const formByStatus = useCallback(
    (status: PlanStatus) => {
      const onSubmitFinished = (outcomes: StopOutcomeDto[]) => {
        mutate({
          status: PlanStatus.Finished,
          outcomes: completeOutcomes(loadedData.stops, outcomes),
        })
      }

      const onSubmitCancel: SubmitHandler<PlanCancelForm> = (data) => {
        mutate({ status: PlanStatus.Canceled, cancellationNote: data.cancellationNote })
      }

      const onSubmitNotCompleted: SubmitHandler<PlanCancelForm> = (data) => {
        mutate({ status: PlanStatus.NotCompleted, cancellationNote: data.cancellationNote })
      }

      const onSubmitOtherStatus = (status: PlanStatus) => {
        mutate({ status })
      }
      switch (status) {
        case 'canceled':
          return <CancelPlan onSubmit={onSubmitCancel} className="mt-6 md:w-1/2" />
        case PlanStatus.NotCompleted:
          return (
            <CancelPlan
              onSubmit={onSubmitNotCompleted}
              className="mt-6 md:w-1/2"
              label={t('statusUpdate.notCompletedReasonLabel')}
              placeholder={t('statusUpdate.notCompletedReasonPlaceholder')}
            />
          )
        case 'finished':
          return planning ? (
            <FinishPlan plan={loadedData} planning={planning} onSubmit={onSubmitFinished} />
          ) : (
            <InlineAlert
              variant="info"
              className="mt-6 md:w-1/2"
              description={t('statusUpdate.finishModuleInactive')}
            />
          )
        default:
          return (
            <Button onClick={() => onSubmitOtherStatus(status)} type="submit" className="mt-10">
              {t('common:actions.save')}
              <MoveRight className="icon-arrow-animate" />
            </Button>
          )
      }
    },
    [loadedData, planning, mutate, t],
  )

  return (
    <>
      <FormPageHeader
        backLink={{
          label: t('statusUpdate.backLabel'),
          link: {
            to: `/plans/$planId`,
            params: { planId },
          },
        }}
        title={<>{t('statusUpdate.title', { date })}</>}
      >
        <p className="flex gap-x-3 mb-5">
          <strong>{t('statusUpdate.currentStatusLabel')}</strong>
          <Badge variant={statusDetails.color} size="lg">
            {statusDetails.label}
          </Badge>
        </p>
        <p>
          {i18n.t(
            planTextKey(planning, 'statusUpdateDescription', 'plan:statusUpdate.description'),
          )}
        </p>
      </FormPageHeader>

      <section className="mt-10">
        {statusOptions.length === 0 ? (
          <p className="text-dark-600 md:w-1/2">{t('statusUpdate.finishedNotice')}</p>
        ) : (
          <>
            <div className="flex flex-col gap-y-6 md:w-1/2">
              <SelectField
                id="status"
                label={t('statusUpdate.statusFieldLabel')}
                placeholder={t('statusUpdate.statusFieldPlaceholder')}
                required
                value={selectedStatus.value}
                description={selectedStatus.description}
                onValueChange={(value) => {
                  // SelectField's onValueChange is string-typed; value is always
                  // one of statusOptions' PlanStatus values.
                  setSelectedStatus(getPlanStatusDetails(value as PlanStatus))
                }}
                options={statusOptions}
              />
            </div>
            {formByStatus(selectedStatus.value)}
            <FormError show={isError} error={error?.message} />
          </>
        )}
      </section>
    </>
  )
}

interface CancelPlanProps {
  onSubmit: SubmitHandler<PlanCancelForm>
  submitLabel?: string
  className?: string
  label?: string
  placeholder?: string
}

export const CancelPlan = ({
  onSubmit,
  submitLabel,
  className = 'md:w-1/2',
  label,
  placeholder,
}: CancelPlanProps) => {
  const { t } = useTranslation(['plan', 'common'])
  const {
    register,
    handleSubmit,
    formState: { isValid, errors },
  } = useForm({
    mode: 'onChange',
    resolver: zodResolver(planCancelSchema),
  })

  return (
    <form className={className} onSubmit={handleSubmit(onSubmit)}>
      <TextareaField
        placeholder={placeholder ?? t('statusUpdate.cancelReasonPlaceholder')}
        label={label ?? t('statusUpdate.cancelReasonLabel')}
        error={errors.cancellationNote?.message}
        required
        {...register('cancellationNote')}
      />

      <Button type="submit" disabled={!isValid} className="mt-10">
        {submitLabel ?? t('common:actions.save')}
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

interface FinishPlanProps {
  plan: PlanInListResponse
  planning: ModulePlanning
  onSubmit: (outcomes: StopOutcomeDto[]) => void
  submitLabel?: string
}

/** The amounts per stop come from the plan module; the core only gates the submit on them. */
export const FinishPlan = ({ plan, planning, onSubmit, submitLabel }: FinishPlanProps) => {
  const { t } = useTranslation('common')
  const [result, setResult] = useState<{ outcomes: StopOutcomeDto[]; valid: boolean }>({
    outcomes: [],
    valid: false,
  })
  // Stable, so an input that reports from an effect does not re-run it on every render.
  const handleChange = useCallback(
    (outcomes: StopOutcomeDto[], valid: boolean) => setResult({ outcomes, valid }),
    [],
  )
  const { OutcomeInput } = planning

  return (
    <form
      className="flex flex-col gap-y-6"
      onSubmit={(event) => {
        event.preventDefault()
        if (result.valid) onSubmit(result.outcomes)
      }}
    >
      <Suspense fallback={<Loading className="mt-6" label={t('state.loadingInline')} />}>
        <OutcomeInput plan={plan} onChange={handleChange} />
      </Suspense>

      <Button type="submit" disabled={!result.valid} className="mt-10">
        {submitLabel ?? t('actions.save')}
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default PlanStatusUpdate
