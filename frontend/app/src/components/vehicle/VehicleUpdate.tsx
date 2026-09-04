import FormPageHeader from '../general/FormPageHeader'
import DeleteSection from '../treecluster/DeleteSection'
import { Can } from '@/lib/auth/Can'
import type { Vehicle } from '@/api/backendApi'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { vehicleQueries } from '@/api/queries'
import { vehicleApi } from '@/api/backendApi'
import { FormProvider, SubmitHandler } from 'react-hook-form'
import { Suspense } from 'react'
import { Loading } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import { VehicleForm } from '@/schema/vehicleSchema'
import FormForVehicle from '../general/form/FormForVehicle'
import UnsavedChangesDialog from '../general/form/UnsavedChangesDialog'
import { useVehicleForm } from '@/hooks/form/useVehicleForm'

interface VehicleUpdateProps {
  vehicleId: string
}

const VehicleUpdate = ({ vehicleId }: VehicleUpdateProps) => {
  const { t } = useTranslation('vehicle')
  const { initForm, loadedData } = useInitFormQuery<Vehicle, VehicleForm>(
    vehicleQueries.detail(vehicleId),
    (data) => ({
      numberPlate: data.numberPlate,
      type: data.type,
      drivingLicense: data.drivingLicense,
      availability: data.availability,
      height: data.height,
      width: data.width,
      length: data.length,
      weight: data.weight,
      model: data.model,
      waterCapacity: data.waterCapacity,
      description: data.description,
    }),
  )
  const { mutate, isError, error, form, navigationBlocker } = useVehicleForm('update', {
    vehicleId,
    initForm,
  })
  const onSubmit: SubmitHandler<VehicleForm> = (data) => {
    mutate(data)
  }

  const handleArchiveVehicle = () => {
    return vehicleApi.archiveVehicle({
      vehicleId,
    })
  }

  return (
    <>
      <FormPageHeader
        backLink={{
          label: t('update.backLabel'),
          link: {
            to: `/vehicles/$vehicleId`,
            params: { vehicleId: vehicleId?.toString() ?? '' },
          },
        }}
        title={<>{t('update.title', { numberPlate: loadedData?.numberPlate })}</>}
      >
        <p className="mb-5">{t('update.description')}</p>
      </FormPageHeader>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForVehicle
            onSubmit={onSubmit}
            displayError={isError}
            errorMessage={error?.message}
          />
        </FormProvider>
      </section>

      <Can permission={['vehicle:delete']}>
        <Suspense
          fallback={<Loading className="mt-20 justify-center" label={t('update.archivingLabel')} />}
        >
          <DeleteSection
            mutationFn={handleArchiveVehicle}
            type="archive"
            entityName={{ key: 'vehicle:entity.nameWithArticle' }}
            redirectUrl={{ to: '/vehicles' }}
            invalidates={['vehicle', 'wateringPlan']}
          />
        </Suspense>
      </Can>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}

export default VehicleUpdate
