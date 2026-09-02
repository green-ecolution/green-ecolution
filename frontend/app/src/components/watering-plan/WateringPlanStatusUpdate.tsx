import { useCallback, useState } from 'react'
import FormPageHeader from '../general/FormPageHeader'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan, WateringPlanUpdate } from '@/api/backendApi'
import { wateringPlanQueries } from '@/api/queries'
import { format } from 'date-fns'
import { Droplet, MoveRight } from 'lucide-react'
import FormError from '../general/form/FormError'
import {
  useWateringPlanStatusDetails,
  useWateringPlanStatusTransitionOptions,
} from '@/hooks/details/useDetailsForWateringPlanStatus'
import { Badge, TextareaField, FormField, SelectField, Button } from '@green-ecolution/ui'
import {
  WateringPlanFinishedForm,
  wateringPlanFinishedSchema,
  WateringPlanCancelForm,
  wateringPlanCancelSchema,
} from '@/schema/wateringPlanSchema'
import { zodResolver } from '@/lib/zodResolver'
import { SubmitHandler, useFieldArray, useForm } from 'react-hook-form'
import { useMutation, useSuspenseQuery } from '@tanstack/react-query'
import { PLAN_STATUS_AGGREGATES, useInvalidateAggregates } from '@/lib/queryInvalidation'
import { wateringPlanApi } from '@/api/backendApi'
import { useNavigate } from '@tanstack/react-router'
import createToast from '@/hooks/createToast'
import { useTranslation } from 'react-i18next'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { toApiError } from '@/lib/apiError'

interface WateringPlanStatusUpdateProps {
  wateringPlanId: string
}

const WateringPlanStatusUpdate = ({ wateringPlanId }: WateringPlanStatusUpdateProps) => {
  const { data: loadedData } = useSuspenseQuery(wateringPlanQueries.detail(wateringPlanId))
  const navigate = useNavigate()
  const invalidate = useInvalidateAggregates()
  const showToast = createToast()
  const { t } = useTranslation(['wateringPlan', 'errors', 'common'])
  const dateLocale = useDateLocale()
  const getWateringPlanStatusDetails = useWateringPlanStatusDetails()
  const statusDetails = getWateringPlanStatusDetails(loadedData.status)
  const statusOptions = useWateringPlanStatusTransitionOptions(loadedData.status)
  const [selectedStatus, setSelectedStatus] = useState(statusDetails)

  const { mutate, isError, error } = useMutation({
    mutationFn: async (wateringPlan: WateringPlanUpdate) => {
      try {
        return await wateringPlanApi.updateWateringPlan({
          wateringPlanId: wateringPlanId,
          wateringPlanUpdateRequest: wateringPlan,
        })
      } catch (error) {
        throw await toApiError(error)
      }
    },

    onSuccess: (data: WateringPlan) => {
      invalidate(PLAN_STATUS_AGGREGATES).catch((error) =>
        console.error('Invalidation after watering plan status update failed:', error),
      )

      navigate({
        to: `/watering-plans/$wateringPlanId`,
        params: { wateringPlanId: data.id.toString() },
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))

      showToast(t('statusUpdate.toastSuccess'))
    },

    onError: (error) => {
      console.error('Error with vehicle mutation:', error)
      // mutationFn already routed this through toApiError, so error.message is
      // the resolved catalog text — the same value FormError renders below.
      showToast(
        t('errors:frame.wateringPlanStatusUpdateFailed', { reason: error.message }),
        'error',
      )
    },
    throwOnError: true,
  })

  const date = format(new Date(loadedData.date), 'dd.MM.yyyy', { locale: dateLocale })

  const formByStatus = useCallback(
    (status: WateringPlanStatus) => {
      const onSubmitFinished: SubmitHandler<WateringPlanFinishedForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.Finished,
          evaluation: data.evaluation,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }

      const onSubmitCancel: SubmitHandler<WateringPlanCancelForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.Canceled,
          cancellationNote: data.cancellationNote,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }

      const onSubmitNotCompleted: SubmitHandler<WateringPlanCancelForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.NotCompleted,
          cancellationNote: data.cancellationNote,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }

      const onSubmitOtherStatus = (status: WateringPlanStatus) => {
        mutate({
          ...loadedData,
          status,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }
      switch (status) {
        case 'canceled':
          return <CancelWateringPlan onSubmit={onSubmitCancel} className="mt-6 md:w-1/2" />
        case WateringPlanStatus.NotCompleted:
          return (
            <CancelWateringPlan
              onSubmit={onSubmitNotCompleted}
              className="mt-6 md:w-1/2"
              label={t('statusUpdate.notCompletedReasonLabel')}
              placeholder={t('statusUpdate.notCompletedReasonPlaceholder')}
            />
          )
        case 'finished':
          return (
            <FinishedWateringPlan
              onSubmit={onSubmitFinished}
              wateringPlanId={wateringPlanId}
              loadedData={loadedData}
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
    [loadedData, wateringPlanId, mutate, t],
  )

  return (
    <>
      <FormPageHeader
        backLink={{
          label: t('statusUpdate.backLabel'),
          link: {
            to: `/watering-plans/$wateringPlanId`,
            params: { wateringPlanId },
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
        <p>{t('statusUpdate.description')}</p>
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
                  // one of statusOptions' WateringPlanStatus values.
                  setSelectedStatus(getWateringPlanStatusDetails(value as WateringPlanStatus))
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
  onSubmit: SubmitHandler<WateringPlanCancelForm>
  submitLabel?: string
  className?: string
  label?: string
  placeholder?: string
}

export const CancelWateringPlan = ({
  onSubmit,
  submitLabel,
  className = 'md:w-1/2',
  label,
  placeholder,
}: CancelPlanProps) => {
  const { t } = useTranslation(['wateringPlan', 'common'])
  const {
    register,
    handleSubmit,
    formState: { isValid, errors },
  } = useForm({
    mode: 'onChange',
    resolver: zodResolver(wateringPlanCancelSchema),
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

interface FinishedPlanProps {
  onSubmit: SubmitHandler<WateringPlanFinishedForm>
  wateringPlanId: string
  loadedData: Pick<WateringPlan, 'treeclusters'>
  submitLabel?: string
}

export const FinishedWateringPlan = ({
  wateringPlanId,
  onSubmit,
  loadedData,
  submitLabel,
}: FinishedPlanProps) => {
  const { t } = useTranslation(['wateringPlan', 'common'])
  const {
    register,
    handleSubmit,
    formState: { isValid },
    control,
  } = useForm({
    mode: 'onChange',
    resolver: zodResolver(wateringPlanFinishedSchema),
    defaultValues: {
      evaluation: loadedData.treeclusters.map((cluster: { treeIds?: string[]; id: string }) => ({
        consumedWater: (cluster.treeIds?.length ?? 1) * 80,
        treeClusterId: cluster.id,
        wateringPlanId: wateringPlanId,
      })),
    },
  })

  const { fields } = useFieldArray({
    control,
    name: 'evaluation',
  })

  return (
    <form className="flex flex-col gap-y-6" onSubmit={handleSubmit(onSubmit)}>
      <fieldset className="mt-6">
        <legend className="block font-semibold text-dark-800 mb-2.5">
          {t('statusUpdate.finished.waterPerClusterLegend')}
        </legend>
        <p className="-mt-2 text-sm text-dark-600 mb-2.5">
          {t('statusUpdate.finished.defaultsHint')}
        </p>
        <ul className="flex flex-col">
          {fields.map((field, index) => {
            const cluster = loadedData.treeclusters[index]
            const treeCount = cluster.treeIds.length
            return (
              <li
                key={field.treeClusterId}
                className="flex items-center justify-between gap-4 border-b border-dark-100 py-3 last:border-0"
              >
                <div className="flex min-w-0 items-center gap-2.5">
                  <span
                    aria-hidden
                    className="flex size-8 shrink-0 items-center justify-center rounded-full bg-green-light-100"
                  >
                    <Droplet className="size-4 text-green-dark" />
                  </span>
                  <div className="min-w-0">
                    <p className="truncate font-medium text-dark">{cluster.name}</p>
                    {treeCount > 0 && (
                      <p className="text-xs tabular-nums text-dark-600">
                        {t('statusUpdate.finished.treeCount', { count: treeCount })}
                      </p>
                    )}
                  </div>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <FormField
                    type="number"
                    label={t('statusUpdate.finished.litersForClusterLabel', { name: cluster.name })}
                    defaultValue={field.consumedWater}
                    className="max-w-28"
                    hideLabel
                    {...register(`evaluation.${index}.consumedWater`)}
                  />
                  <span className="text-sm text-dark-600">
                    {t('statusUpdate.finished.litersUnitLabel')}
                  </span>
                </div>
              </li>
            )
          })}
        </ul>
      </fieldset>

      <Button type="submit" disabled={!isValid} className="mt-10">
        {submitLabel ?? t('common:actions.save')}
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default WateringPlanStatusUpdate
