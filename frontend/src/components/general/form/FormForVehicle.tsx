import { VehicleForm } from '@/schema/vehicleSchema'
import FormError from './FormError'
import PrimaryButton from '../buttons/PrimaryButton'
import Input from './types/Input'
import Textarea from './types/Textarea'
import Select from './types/Select'
import { VehicleTypeOptions } from '@/hooks/details/useDetailsForVehicleType'
import { DrivingLicenseOptions } from '@/hooks/details/useDetailsForDrivingLicense'
import { VehicleStatusOptions } from '@/hooks/details/useDetailsForVehicleStatus'
import { SubmitHandler, useFormContext } from 'react-hook-form'

interface FormForVehicleProps {
  displayError: boolean
  errorMessage?: string
  onSubmit: SubmitHandler<VehicleForm>
}

const FormForVehicle = (props: FormForVehicleProps) => {
  const {
    register,
    handleSubmit,
    formState: { errors, isValid },
  } = useFormContext<VehicleForm>()

  const translateNum = (e: React.ChangeEvent<HTMLInputElement>) =>
    (e.target.value = e.target.value.replace(',', '.'))

  return (
    <form
      className="space-y-6 lg:space-y-0 lg:grid lg:grid-cols-2 lg:gap-y-6 lg:gap-x-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <Input
        placeholder="Kennzeichen"
        label="Kennzeichen"
        required
        error={errors.numberPlate?.message}
        {...register('numberPlate')}
      />
      <Input
        placeholder="Fahrzeugmodell"
        label="Fahrzeugmodell"
        required
        error={errors.model?.message}
        {...register('model')}
      />
      <Select
        options={VehicleTypeOptions}
        placeholder="Fahrzeugtyp"
        label="Fahrzeugtyp"
        required
        error={errors.type?.message}
        {...register('type')}
      />
      <Input
        placeholder="Wasserkapazität"
        label="Wasserkapazität"
        type="number"
        required
        error={errors.waterCapacity?.message}
        {...register('waterCapacity')}
      />
      <Select
        options={VehicleStatusOptions}
        placeholder="Aktueller Fahrzeugstatus"
        label="Aktueller Status"
        required
        error={errors.status?.message}
        {...register('status')}
      />
      <Select
        options={DrivingLicenseOptions}
        placeholder="Wählen Sie eine Führerscheinklasse aus"
        label="Führerscheinklasse"
        required
        error={errors.drivingLicense?.message}
        {...register('drivingLicense')}
      />
      <Input
        placeholder="Höhe des Fahrzeugs"
        label="Höhe des Fahrzeugs (in Metern)"
        step="0.1"
        required
        error={errors.height?.message}
        {...register('height', {
          onChange: translateNum,
        })}
      />
      <Input
        placeholder="Breite des Fahrzeugs"
        label="Breite des Fahrzeugs (in Metern)"
        step="0.1"
        required
        error={errors.width?.message}
        {...register('width', {
          onChange: translateNum,
        })}
      />
      <Input
        placeholder="Länge des Fahrzeugs"
        label="Länge des Fahrzeugs (in Metern)"
        step="0.1"
        required
        error={errors.length?.message}
        {...register('length', {
          onChange: translateNum,
        })}
      />
      <Input
        placeholder="Gewicht des Fahrzeugs"
        label="Gewicht des Fahrzeugs (in Tonnen)"
        step="0.1"
        required
        error={errors.weight?.message}
        {...register('weight', {
          onChange: translateNum,
        })}
      />
      <Textarea
        placeholder="Hier ist Platz für Notizen"
        label="Kurze Beschreibung"
        error={errors.description?.message}
        {...register('description')}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <PrimaryButton
        type="submit"
        label="Speichern"
        className="mt-10 lg:col-span-full lg:w-fit"
        disabled={!isValid}
      />
    </form>
  )
}

export default FormForVehicle
