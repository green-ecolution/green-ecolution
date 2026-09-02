import { useSuspenseQuery } from '@tanstack/react-query'
import FormPageHeader from '../general/FormPageHeader'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { userQueries, vehicleQueries, wateringPlanQueries } from '@/api/queries'
import { format } from 'date-fns'
import FormForWateringPlan from '../general/form/FormForWateringPlan'
import { useNavigate } from '@tanstack/react-router'
import { Route } from '@/routes'
import useStore from '@/store/store'
import GeneralLink from '../general/links/GeneralLink'
import { showWateringPlanStatusButton } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { Loading } from '@green-ecolution/ui'
import { Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import DeleteSection from '../treecluster/DeleteSection'
import { Can } from '@/lib/auth/Can'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { wateringPlanApi } from '@/api/backendApi'
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
  const { t } = useTranslation(['wateringPlan', 'common'])
  const dateLocale = useDateLocale()
  const canUpdateStatus = useHasPermission(['watering_plan:update'])
  const draft = useWateringPlanDraft<WateringPlanForm>('update')
  const { initForm, loadedData } = useInitFormQuery(
    wateringPlanQueries.detail(wateringPlanId),
    (data) =>
      draft.data ?? {
        date: new Date(data.date),
        description: data.description,
        transporterId: data.transporter.id,
        trailerId: data.trailer?.id,
        clusterIds: data.treeclusters.map((cluster) => cluster.id),
        status: data.status,
        driverIds: data.userIds,
        startPointName: data.startPointName ?? '',
      },
  )
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useWateringPlanForm(
    'update',
    {
      wateringPlanId,
      initForm,
    },
  )

  const navigate = useNavigate({ from: Route.fullPath })
  const date = loadedData?.date
    ? format(new Date(loadedData?.date), 'dd.MM.yyyy', { locale: dateLocale })
    : t('common:state.noData')

  const { data: users } = useSuspenseQuery(userQueries.list({ page: 1, perPage: 100 }))
  const { data: trailers } = useSuspenseQuery(vehicleQueries.list())
  const { data: transporters } = useSuspenseQuery(vehicleQueries.list())

  const onSubmit: SubmitHandler<WateringPlanForm> = (data) => {
    mutate({
      ...data,
      date: data.date.toISOString(),
      trailerId: data.trailerId && data.trailerId !== '-1' ? data.trailerId : undefined,
      treeClusterIds: data.clusterIds,
      userIds: data.driverIds,
      startPointName: data.startPointName,
      cancellationNote: '', // TODO: why cancel note in update ???
    })
  }

  const mapCenter = useStore((state) => state.mapCenter)
  const mapZoom = useStore((state) => state.mapZoom)
  const mapPosition = { lat: mapCenter[0], lng: mapCenter[1], zoom: mapZoom }

  const navigateToClusterSelect = () => {
    saveDraft()
    navigate({
      to: '/map/watering-plan/select/cluster',
      search: {
        lat: mapPosition.lat,
        lng: mapPosition.lng,
        zoom: mapPosition.zoom,
        wateringPlanId,
        trailerId: form.getValues('trailerId'),
        transporterId: form.getValues('transporterId'),
        clusterIds: form.getValues('clusterIds'),
        formType: 'update',
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const handleDeleteWateringPlan = () => {
    return wateringPlanApi.deleteWateringPlan({
      wateringPlanId,
    })
  }

  return (
    <>
      <FormPageHeader
        backLink={{
          label: t('update.backLabel'),
          link: {
            to: `/watering-plans/$wateringPlanId`,
            params: { wateringPlanId: wateringPlanId?.toString() },
          },
        }}
        title={<>{t('update.title', { date })}</>}
      >
        <p>{t('update.description')}</p>
        {canUpdateStatus && showWateringPlanStatusButton(loadedData) && (
          <p className="mt-5 flex flex-wrap gap-x-4">
            {t('update.statusEditNotice')}
            <GeneralLink
              link={{
                to: `/watering-plans/$wateringPlanId/status/edit`,
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
            onAddCluster={navigateToClusterSelect}
            errorMessage={error?.message}
            onBlur={saveDraft}
          />
        </FormProvider>
      </section>

      <Can permission={['watering_plan:delete']}>
        <Suspense
          fallback={<Loading className="mt-20 justify-center" label={t('update.deletingLabel')} />}
        >
          <DeleteSection
            mutationFn={handleDeleteWateringPlan}
            entityName={{ key: 'wateringPlan:entity.nameWithArticle' }}
            redirectUrl={{ to: '/watering-plans' }}
            invalidates={['wateringPlan', 'evaluation']}
          />
        </Suspense>
      </Can>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}

export default WateringPlanUpdate
