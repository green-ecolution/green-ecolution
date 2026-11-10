import FormForVehicle from '@/components/general/form/FormForVehicle'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import BackLink from '@/components/general/links/BackLink'
import { VehicleForm } from '@/schema/vehicleSchema'
import { createFileRoute } from '@tanstack/react-router'
import { NEW_VEHICLE_DEFAULTS, useVehicleForm } from '@/hooks/form/useVehicleForm'
import { FormProvider } from 'react-hook-form'
import { useTranslation } from 'react-i18next'

export const Route = createFileRoute('/_protected/vehicles/_formular/new/')({
  component: NewVehicle,
})

function NewVehicle() {
  const { t } = useTranslation('vehicle')
  const { mutate, isError, error, form, navigationBlocker } = useVehicleForm('create', {
    initForm: NEW_VEHICLE_DEFAULTS,
  })
  const onSubmit = (data: VehicleForm) => {
    mutate({ ...data })
  }

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <BackLink link={{ to: '/vehicles' }} label={t('create.backLabel')} />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('create.title')}
        </h1>
        <p className="mb-5">{t('create.description')}</p>
      </article>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForVehicle
            onSubmit={onSubmit}
            displayError={isError}
            errorMessage={error?.message}
          />
        </FormProvider>
      </section>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </div>
  )
}
