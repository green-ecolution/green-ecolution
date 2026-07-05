import FormPageHeader from '../general/FormPageHeader'
import DeleteSection from '../treecluster/DeleteSection'
import type { Vehicle } from '@/api/backendApi'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { vehicleIdQuery } from '@/api/queries'
import { vehicleApi } from '@/api/backendApi'
import { FormProvider, SubmitHandler } from 'react-hook-form'
import { Suspense } from 'react'
import { Loading } from '@green-ecolution/ui'
import { VehicleForm } from '@/schema/vehicleSchema'
import FormForVehicle from '../general/form/FormForVehicle'
import UnsavedChangesDialog from '../general/form/UnsavedChangesDialog'
import { useVehicleForm } from '@/hooks/form/useVehicleForm'

interface VehicleUpdateProps {
  vehicleId: string
}

const VehicleUpdate = ({ vehicleId }: VehicleUpdateProps) => {
  const { initForm, loadedData } = useInitFormQuery<Vehicle, VehicleForm>(
    vehicleIdQuery(vehicleId),
    (data) => ({
      numberPlate: data.numberPlate,
      type: data.type,
      drivingLicense: data.drivingLicense,
      status: data.status,
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
          label: 'Zurück zur Fahrzeugübersicht',
          link: {
            to: `/vehicles/$vehicleId`,
            params: { vehicleId: vehicleId?.toString() ?? '' },
          },
        }}
        title={<>Fahrzeug {loadedData?.numberPlate} bearbeiten</>}
      >
        <p className="mb-5">Hier können Sie das Fahrzeug bearbeiten.</p>
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

      <Suspense
        fallback={<Loading className="mt-20 justify-center" label="Das Fahrzeug wird gelöscht" />}
      >
        <DeleteSection
          mutationFn={handleArchiveVehicle}
          type="archive"
          entityName="das Fahrzeug"
          redirectUrl={{ to: '/vehicles' }}
        />
      </Suspense>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}

export default VehicleUpdate
