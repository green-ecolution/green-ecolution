import { VehicleForm } from '@/schema/vehicleSchema'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import { FormField, TextareaField, SelectField } from '@green-ecolution/ui'
import { VehicleTypeOptions } from '@/hooks/details/useDetailsForVehicleType'
import { DrivingLicenseOptions } from '@/hooks/details/useDetailsForDrivingLicense'
import { VehicleStatusOptions } from '@/hooks/details/useDetailsForVehicleStatus'
import { Controller, SubmitHandler, useFormContext, useFormState } from 'react-hook-form'

interface FormForVehicleProps {
  displayError: boolean
  errorMessage?: string
  onSubmit: SubmitHandler<VehicleForm>
}

const FormForVehicle = (props: FormForVehicleProps) => {
  const { register, handleSubmit, control } = useFormContext<VehicleForm>()
  const { isValid, errors } = useFormState({ control })

  const translateNum = (e: React.ChangeEvent<HTMLInputElement>) =>
    (e.target.value = e.target.value.replace(',', '.'))

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-y-6 lg:gap-x-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <FormField
        placeholder="Kennzeichen"
        label="Kennzeichen"
        required
        error={errors.numberPlate?.message}
        {...register('numberPlate')}
      />
      <FormField
        placeholder="Fahrzeugmodell"
        label="Fahrzeugmodell"
        required
        error={errors.model?.message}
        {...register('model')}
      />
      <Controller
        name="type"
        control={control}
        render={({ field }) => (
          <SelectField
            id="type"
            label="Fahrzeugtyp"
            placeholder="Fahrzeugtyp"
            required
            value={field.value}
            onValueChange={field.onChange}
            error={errors.type?.message}
            options={VehicleTypeOptions}
          />
        )}
      />
      <FormField
        placeholder="Wasserkapazität"
        label="Wasserkapazität"
        type="number"
        required
        error={errors.waterCapacity?.message}
        {...register('waterCapacity')}
      />
      <Controller
        name="status"
        control={control}
        render={({ field }) => (
          <SelectField
            id="status"
            label="Aktueller Status"
            placeholder="Aktueller Fahrzeugstatus"
            required
            value={field.value}
            onValueChange={field.onChange}
            error={errors.status?.message}
            options={VehicleStatusOptions}
          />
        )}
      />
      <Controller
        name="drivingLicense"
        control={control}
        render={({ field }) => (
          <SelectField
            id="drivingLicense"
            label="Führerscheinklasse"
            placeholder="Wählen Sie eine Führerscheinklasse aus"
            required
            value={field.value}
            onValueChange={field.onChange}
            error={errors.drivingLicense?.message}
            options={DrivingLicenseOptions}
          />
        )}
      />
      <FormField
        placeholder="Höhe des Fahrzeugs"
        label="Höhe des Fahrzeugs (in Metern)"
        step="0.1"
        required
        error={errors.height?.message}
        {...register('height', {
          onChange: translateNum,
        })}
      />
      <FormField
        placeholder="Breite des Fahrzeugs"
        label="Breite des Fahrzeugs (in Metern)"
        step="0.1"
        required
        error={errors.width?.message}
        {...register('width', {
          onChange: translateNum,
        })}
      />
      <FormField
        placeholder="Länge des Fahrzeugs"
        label="Länge des Fahrzeugs (in Metern)"
        step="0.1"
        required
        error={errors.length?.message}
        {...register('length', {
          onChange: translateNum,
        })}
      />
      <FormField
        placeholder="Gewicht des Fahrzeugs"
        label="Gewicht des Fahrzeugs (in Tonnen)"
        step="0.1"
        required
        error={errors.weight?.message}
        {...register('weight', {
          onChange: translateNum,
        })}
      />
      <TextareaField
        placeholder="Hier ist Platz für Notizen"
        label="Kurze Beschreibung"
        error={errors.description?.message}
        {...register('description')}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton disabled={!isValid} />
    </form>
  )
}

export default FormForVehicle
