import { useSuspenseQuery } from '@tanstack/react-query'
import FormPageHeader from '../general/FormPageHeader'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { userQueries, vehicleQueries, wateringPlanQueries } from '@/api/queries'
import { format } from 'date-fns'
import FormForWateringPlan from '../general/form/FormForWateringPlan'
import GeneralLink from '../general/links/GeneralLink'
import { showWateringPlanStatusButton } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { Loading } from '@green-ecolution/ui'
import { Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import DeleteSection from '../general/DeleteSection'
import { planApi } from '@/api/backendApi'
import { planStatusQueryRoots } from '@/lib/queryInvalidation'
import { planningManifests, planTextKey } from '@/modules/planning'
import { useActiveModules } from '@/modules/useActiveModules'
import { usePlanPermission } from '@/modules/usePlanPermission'
import { useWateringPlanForm } from '@/hooks/form/useWateringPlanForm'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { FormProvider, SubmitHandler } from 'react-hook-form'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import UnsavedChangesDialog from '../general/form/UnsavedChangesDialog'

interface WateringPlanUpdateProps {
  wateringPlanId: string
}

const WateringPlanUpdate = ({ wateringPlanId }: WateringPlanUpdateProps) => {
  const { t, i18n } = useTranslation(['wateringPlan', 'common'])
  const dateLocale = useDateLocale()
  const draft = useWateringPlanDraft<WateringPlanForm>('update')
  const { initForm, loadedData } = useInitFormQuery(
    wateringPlanQueries.detail(wateringPlanId),
    (data) =>
      draft.data ?? {
        date: new Date(data.date),
        description: data.description,
        transporterId: data.transporter.id,
        trailerId: data.trailer?.id,
        transporterTankUsed: data.transporterCapacityUsed,
        // Every stop, also those the reader cannot see: the stop picker only
        // removes what the user removes, so a save keeps the hidden ones.
        stopIds: data.stops,
        module: data.module,
        status: data.status,
        driverIds: data.userIds,
        startPointName: data.startPointName ?? '',
      },
  )
  const { manifests } = useActiveModules()
  const planManifest = planningManifests(manifests).find(({ key }) => key === loadedData.module)
  const planning = planManifest?.planning
  const canUpdateStatus = usePlanPermission(planning, 'update')
  const canDelete = usePlanPermission(planning, 'delete')
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useWateringPlanForm(
    'update',
    {
      wateringPlanId,
      initForm,
      planManifest,
    },
  )

  const date = loadedData?.date
    ? format(new Date(loadedData?.date), 'dd.MM.yyyy', { locale: dateLocale })
    : t('common:state.noData')

  const { data: users } = useSuspenseQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: trailers } = useSuspenseQuery(vehicleQueries.list())
  const { data: transporters } = useSuspenseQuery(vehicleQueries.list())

  const onSubmit: SubmitHandler<WateringPlanForm> = (data) => mutate(data)

  const handleDeleteWateringPlan = () => {
    return planApi.deletePlan({ planId: wateringPlanId })
  }

  return (
    <>
      <FormPageHeader
        backLink={{
          label: t('update.backLabel'),
          link: {
            to: `/plans/$wateringPlanId`,
            params: { wateringPlanId: wateringPlanId?.toString() },
          },
        }}
        title={<>{t('update.title', { date })}</>}
      >
        <p>
          {i18n.t(planTextKey(planning, 'updateDescription', 'wateringPlan:update.description'))}
        </p>
        {canUpdateStatus && showWateringPlanStatusButton(loadedData) && (
          <p className="mt-5 flex flex-wrap gap-x-4">
            {t('update.statusEditNotice')}
            <GeneralLink
              link={{
                to: `/plans/$wateringPlanId/status/edit`,
                params: { wateringPlanId: String(loadedData.id) },
              }}
              label={t('detail.updateStatusLabel')}
            />
          </p>
        )}
      </FormPageHeader>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForWateringPlan
            displayError={isError}
            onSubmit={onSubmit}
            users={users.data}
            trailers={trailers.data}
            transporters={transporters.data}
            formType="update"
            planId={wateringPlanId}
            organizationId={loadedData.organizationId}
            saveDraft={saveDraft}
            errorMessage={error?.message}
            onBlur={saveDraft}
          />
        </FormProvider>
      </section>

      {canDelete && (
        <Suspense
          fallback={<Loading className="mt-20 justify-center" label={t('update.deletingLabel')} />}
        >
          <DeleteSection
            mutationFn={handleDeleteWateringPlan}
            entityName={{ key: 'wateringPlan:entity.nameWithArticle' }}
            redirectUrl={{ to: '/plans' }}
            invalidates={planStatusQueryRoots(planning)}
          />
        </Suspense>
      )}

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}

export default WateringPlanUpdate
