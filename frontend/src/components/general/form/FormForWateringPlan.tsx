import PrimaryButton from '../buttons/PrimaryButton'
import Input from './types/Input'
import Select from './types/Select'
import Textarea from './types/Textarea'
import FormError from './FormError'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { User, Vehicle } from '@green-ecolution/backend-client'
import SelectEntities from './types/SelectEntities'
import { getDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { Controller, SubmitHandler, useFormContext } from 'react-hook-form'

interface FormForWateringPlanProps {
  displayError: boolean
  errorMessage?: string
  transporters: Vehicle[]
  trailers: Vehicle[]
  users: User[]
  onAddCluster: () => void
  onSubmit: SubmitHandler<WateringPlanForm>
}

const FormForWateringPlan = (props: FormForWateringPlanProps) => {
  const {
    register,
    handleSubmit,
    control,
    formState: { isValid, errors },
  } = useFormContext<WateringPlanForm>()

  const getDrivingLicensesString = (user: User) => {
    if (!user.drivingLicenses || user.drivingLicenses.length === 0) {
      return 'Keinen Führerschein'
    }

    return user.drivingLicenses
      .map((drivingLicense) => getDrivingLicenseDetails(drivingLicense).label)
      .join(', ')
  }

  return (
    <form
      className="space-y-6 lg:space-y-0 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <div className="space-y-6">
        <Controller
          control={control}
          name="date"
          render={({ field: { value, onChange } }) => (
            <Input
              label="Datum des Einsatzes"
              error={errors.date?.message}
              required
              type="date"
              onChange={onChange}
              value={new Date(value).toISOString().split('T')[0]}
            />
          )}
        />
        <Select
          options={[
            { label: 'Kein Fahrzeug', value: '-1' },
            ...props.transporters.map((transporter) => ({
              label: `${transporter.numberPlate.toString()} · ${getDrivingLicenseDetails(transporter.drivingLicense).label}`,
              value: transporter.id.toString(),
            })),
          ]}
          placeholder="Wählen Sie ein Fahrzeug aus"
          label="Verknüpftes Fahrzeug"
          required
          error={errors.transporterId?.message}
          {...register('transporterId')}
        />
        <Select
          options={[
            { label: 'Keinen Anhänger', value: '-1' },
            ...props.trailers.map((trailer) => ({
              label: `${trailer.numberPlate.toString()} · ${getDrivingLicenseDetails(trailer.drivingLicense).label}`,
              value: trailer.id.toString(),
            })),
          ]}
          placeholder="Wählen Sie einen Anhänger aus, sofern vorhanden"
          label="Verknüpfter Anhänger"
          error={errors.trailerId?.message}
          {...register('trailerId')}
        />
        <Select
          options={[
            ...props.users.map((user) => ({
              label: `${user.firstName} ${user.lastName} · ${getDrivingLicensesString(user)}`,
              value: user.id,
            })),
          ]}
          multiple
          placeholder="Wählen Sie Mitarbeitende aus"
          label="Verknüpfte Mitarbeitende"
          description="Indem Sie die Taste »Shift« gedrückt halten, können Sie eine Mehrauswahl tätigen."
          required
          error={errors.driverIds?.message}
          {...register('driverIds')}
        />
        <Textarea
          placeholder="Hier ist Platz für Notizen"
          label="Kurze Beschreibung"
          error={errors.description?.message}
          {...register('description')}
        />
      </div>

      <Controller
        control={control}
        name="cluserIds"
        render={({ field: { value, onChange } }) => (
          <SelectEntities
            // eslint-disable-next-line
            onDelete={() => {}} // TODO: remove
            onChange={onChange}
            entityIds={value}
            onAdd={props.onAddCluster}
            type="tree"
            label="Bäume"
          />
        )}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <PrimaryButton
        type="submit"
        label="Speichern"
        disabled={!isValid}
        className="mt-10 lg:col-span-full lg:w-fit"
      />
    </form>
  )
}

export default FormForWateringPlan
