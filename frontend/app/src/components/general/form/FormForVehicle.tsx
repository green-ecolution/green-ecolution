import { VehicleForm } from '@/schema/vehicleSchema'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import { FormField, TextareaField, SelectField } from '@green-ecolution/ui'
import { useVehicleTypeOptions } from '@/hooks/details/useDetailsForVehicleType'
import { useDrivingLicenseOptions } from '@/hooks/details/useDetailsForDrivingLicense'
import { useVehicleAvailabilityOptions } from '@/hooks/details/useDetailsForVehicleStatus'
import { Controller, SubmitHandler, useFormContext, useFormState } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { parseDecimalInput } from '@/lib/utils'

interface FormForVehicleProps {
  displayError: boolean
  errorMessage?: string
  onSubmit: SubmitHandler<VehicleForm>
}

// The API rejects strings for these fields, so the raw input has to be parsed
// on the way into the form state, not on the way out.
const asNumber = { setValueAs: parseDecimalInput } as const

const FormForVehicle = (props: FormForVehicleProps) => {
  const { t } = useTranslation(['vehicle', 'common'])
  const { register, handleSubmit, control } = useFormContext<VehicleForm>()
  const { isValid, errors } = useFormState({ control })
  const vehicleTypeOptions = useVehicleTypeOptions()
  const vehicleAvailabilityOptions = useVehicleAvailabilityOptions()
  const drivingLicenseOptions = useDrivingLicenseOptions()

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-y-6 lg:gap-x-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <FormField
        placeholder={t('form.numberPlatePlaceholder')}
        label={t('form.numberPlateLabel')}
        required
        error={errors.numberPlate?.message}
        {...register('numberPlate')}
      />
      <FormField
        placeholder={t('form.modelPlaceholder')}
        label={t('form.modelLabel')}
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
            label={t('form.typeLabel')}
            placeholder={t('form.typePlaceholder')}
            required
            value={field.value}
            onValueChange={field.onChange}
            error={errors.type?.message}
            options={vehicleTypeOptions}
          />
        )}
      />
      <FormField
        placeholder={t('form.waterCapacityPlaceholder')}
        label={t('form.waterCapacityLabel')}
        type="number"
        required
        error={errors.waterCapacity?.message}
        {...register('waterCapacity', asNumber)}
      />
      <Controller
        name="availability"
        control={control}
        render={({ field }) => (
          <SelectField
            id="availability"
            label={t('form.availabilityLabel')}
            placeholder={t('form.availabilityPlaceholder')}
            required
            value={field.value}
            onValueChange={field.onChange}
            error={errors.availability?.message}
            options={vehicleAvailabilityOptions}
          />
        )}
      />
      <Controller
        name="drivingLicense"
        control={control}
        render={({ field }) => (
          <SelectField
            id="drivingLicense"
            label={t('form.drivingLicenseLabel')}
            placeholder={t('form.drivingLicensePlaceholder')}
            required
            value={field.value}
            onValueChange={field.onChange}
            error={errors.drivingLicense?.message}
            options={drivingLicenseOptions}
          />
        )}
      />
      <FormField
        placeholder={t('form.heightPlaceholder')}
        label={t('form.heightLabel')}
        step="0.1"
        required
        error={errors.height?.message}
        {...register('height', asNumber)}
      />
      <FormField
        placeholder={t('form.widthPlaceholder')}
        label={t('form.widthLabel')}
        step="0.1"
        required
        error={errors.width?.message}
        {...register('width', asNumber)}
      />
      <FormField
        placeholder={t('form.lengthPlaceholder')}
        label={t('form.lengthLabel')}
        step="0.1"
        required
        error={errors.length?.message}
        {...register('length', asNumber)}
      />
      <FormField
        placeholder={t('form.weightPlaceholder')}
        label={t('form.weightLabel')}
        step="0.1"
        required
        error={errors.weight?.message}
        {...register('weight', asNumber)}
      />
      <TextareaField
        placeholder={t('common:form.notesPlaceholder')}
        label={t('common:form.shortDescriptionLabel')}
        error={errors.description?.message}
        {...register('description')}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton disabled={!isValid} />
    </form>
  )
}

export default FormForVehicle
