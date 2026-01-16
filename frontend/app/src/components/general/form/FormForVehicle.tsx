import { VehicleForm } from '@/schema/vehicleSchema'
import FormError from './FormError'
import { MoveRight } from 'lucide-react'
import {
  FormField,
  TextareaField,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Button,
} from '@green-ecolution/ui'
import { VehicleTypeOptions } from '@/hooks/details/useDetailsForVehicleType'
import { DrivingLicenseOptions } from '@/hooks/details/useDetailsForDrivingLicense'
import { VehicleStatusOptions } from '@/hooks/details/useDetailsForVehicleStatus'
import { Controller, SubmitHandler, useFormContext } from 'react-hook-form'

interface FormForVehicleProps {
  displayError: boolean
  errorMessage?: string
  onSubmit: SubmitHandler<VehicleForm>
}

const FormForVehicle = (props: FormForVehicleProps) => {
  const {
    register,
    handleSubmit,
    control,
    formState: { errors, isValid },
  } = useFormContext<VehicleForm>()

  const translateNum = (e: React.ChangeEvent<HTMLInputElement>) =>
    (e.target.value = e.target.value.replace(',', '.'))

  return (
    <form
      className="space-y-6 lg:space-y-0 lg:grid lg:grid-cols-2 lg:gap-y-6 lg:gap-x-11"
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
          <div className="space-y-2">
            <Label htmlFor="type">
              Fahrzeugtyp
              <span className="text-destructive ml-1">*</span>
            </Label>
            <Select value={field.value} onValueChange={field.onChange}>
              <SelectTrigger id="type">
                <SelectValue placeholder="Fahrzeugtyp" />
              </SelectTrigger>
              <SelectContent>
                {VehicleTypeOptions.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {errors.type?.message && (
              <p className="text-sm text-destructive">{errors.type.message}</p>
            )}
          </div>
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
          <div className="space-y-2">
            <Label htmlFor="status">
              Aktueller Status
              <span className="text-destructive ml-1">*</span>
            </Label>
            <Select value={field.value} onValueChange={field.onChange}>
              <SelectTrigger id="status">
                <SelectValue placeholder="Aktueller Fahrzeugstatus" />
              </SelectTrigger>
              <SelectContent>
                {VehicleStatusOptions.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {errors.status?.message && (
              <p className="text-sm text-destructive">{errors.status.message}</p>
            )}
          </div>
        )}
      />
      <Controller
        name="drivingLicense"
        control={control}
        render={({ field }) => (
          <div className="space-y-2">
            <Label htmlFor="drivingLicense">
              Führerscheinklasse
              <span className="text-destructive ml-1">*</span>
            </Label>
            <Select value={field.value} onValueChange={field.onChange}>
              <SelectTrigger id="drivingLicense">
                <SelectValue placeholder="Wählen Sie eine Führerscheinklasse aus" />
              </SelectTrigger>
              <SelectContent>
                {DrivingLicenseOptions.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {errors.drivingLicense?.message && (
              <p className="text-sm text-destructive">{errors.drivingLicense.message}</p>
            )}
          </div>
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

      <Button type="submit" className="mt-10 lg:col-span-full lg:w-fit" disabled={!isValid}>
        Speichern
        <MoveRight />
      </Button>
    </form>
  )
}

export default FormForVehicle
