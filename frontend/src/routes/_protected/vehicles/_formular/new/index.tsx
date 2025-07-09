import { VehicleType, DrivingLicense, VehicleStatus } from '@green-ecolution/backend-client'
import FormForVehicle from '@/components/general/form/FormForVehicle'
import BackLink from '@/components/general/links/BackLink'
import { VehicleForm } from '@/schema/vehicleSchema'
import useFormStore from '@/store/form/useFormStore'
import { createFileRoute } from '@tanstack/react-router'
import { useVehicleForm } from '@/hooks/form/useVehicleForm'
import { DefaultValues, FormProvider } from 'react-hook-form'

export const Route = createFileRoute('/_protected/vehicles/_formular/new/')({
  beforeLoad: () => {
    useFormStore.getState().setType('new')
  },
  component: NewVehicle,
})

const defaultForm: DefaultValues<VehicleForm> = {
  numberPlate: '',
  type: VehicleType.VehicleTypeTransporter,
  drivingLicense: DrivingLicense.DrivingLicenseB,
  status: VehicleStatus.VehicleStatusUnknown,
  height: 2.5,
  width: 2,
  length: 6,
  weight: 3.5,
  waterCapacity: 300,
}

function NewVehicle() {
  const { mutate, isError, error, form } = useVehicleForm('create', { initForm: defaultForm })
  const onSubmit = (data: VehicleForm) => {
    mutate({ ...data })
  }

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <BackLink link={{ to: '/vehicles' }} label="Zu allen Fahrzeugen" />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Neues Fahrzeug erstellen
        </h1>
        <p className="mb-5">
          In dieser Ansicht können Sie ein neues Fahrzeug anlegen. Bitte beachten Sie, dass jedes
          Fahrzeug ein eindeutiges Kennzeichen besitzen muss, da keine doppelten Kennzeichen erlaubt
          sind. Zusätzlich müssen die Abmessungen des Fahrzeugs hinterlegt werden, damit das
          Navigationssystem bei einer Bewässerungsfahrt ermitteln kann, welche Strecken für das
          Fahrzeug befahrbar sind.
        </p>
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
    </div>
  )
}
