import { VehicleForm } from '@/schema/vehicleSchema'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import {
  FormField,
  TextareaField,
  SelectField,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  Label,
  cn,
} from '@green-ecolution/ui'
import { FORM_CAPACITY_KINDS } from '@/hooks/form/useVehicleForm'
import { useActiveModules } from '@/modules/useActiveModules'
import { capacityDisplay } from '@/lib/capacity'
import { useVehicleTypeOptions } from '@/hooks/details/useDetailsForVehicleType'
import { useDrivingLicenseOptions } from '@/hooks/details/useDetailsForDrivingLicense'
import { useVehicleAvailabilityOptions } from '@/hooks/details/useDetailsForVehicleStatus'
import { Controller, SubmitHandler, useFormContext, useFormState } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { parseDecimalInput } from '@/lib/utils'

type CapacityKind = (typeof FORM_CAPACITY_KINDS)[number]

const EMPTY_CAPACITIES_KEY = 'vehicle.capacities.empty'

interface FormForVehicleProps {
  displayError: boolean
  errorMessage?: string
  onSubmit: SubmitHandler<VehicleForm>
}

// The API rejects strings for these fields, so the raw input has to be parsed
// on the way into the form state, not on the way out.
const asNumber = { setValueAs: parseDecimalInput } as const

// An empty optional field must reach the validator as "not set", not as an empty string.
const asOptionalNumber = {
  setValueAs: (value: unknown) => (value === '' ? undefined : parseDecimalInput(value)),
} as const

const CAPACITY_FIELD = {
  water: 'waterCapacity',
  cargo_volume: 'cargoVolumeCapacity',
} as const

const CapacityField = ({ kind, modules }: { kind: CapacityKind; modules: string[] }) => {
  const { t } = useTranslation('vehicle')
  const { register, control } = useFormContext<VehicleForm>()
  const { errors } = useFormState({ control })
  const name = CAPACITY_FIELD[kind]
  const id = `vehicle-capacity-${kind}`
  const Icon = capacityDisplay[kind].icon
  const error = errors[name]
  // The "at least one" error is reported on the water field but belongs to the group.
  const ownError = error?.type === EMPTY_CAPACITIES_KEY ? undefined : error?.message
  const describedBy = [modules.length > 0 ? `${id}-modules` : null, ownError ? `${id}-error` : null]
    .filter(Boolean)
    .join(' ')

  return (
    <div className="flex flex-col gap-y-2">
      <Label htmlFor={id} className={cn('flex items-center gap-2', ownError && 'text-destructive')}>
        <Icon className="size-4" aria-hidden />
        {t(`form.capacity.${kind}.label`)}
      </Label>
      <InputGroup>
        <InputGroupInput
          id={id}
          type="number"
          placeholder={t(`form.capacity.${kind}.placeholder`)}
          aria-invalid={!!ownError}
          aria-describedby={describedBy || undefined}
          {...register(name, asOptionalNumber)}
        />
        <InputGroupAddon align="inline-end">
          <InputGroupText>{t('form.capacity.unit')}</InputGroupText>
        </InputGroupAddon>
      </InputGroup>
      {modules.length > 0 && (
        <p id={`${id}-modules`} className="text-sm text-muted-foreground">
          {t('form.capacity.usedBy', { label: modules.join(', ') })}
        </p>
      )}
      {ownError && (
        <p
          id={`${id}-error`}
          role="alert"
          aria-live="assertive"
          className="text-sm text-destructive"
        >
          {ownError}
        </p>
      )}
    </div>
  )
}

const FormForVehicle = (props: FormForVehicleProps) => {
  const { t, i18n } = useTranslation(['vehicle', 'common'])
  const { register, handleSubmit, control } = useFormContext<VehicleForm>()
  const { errors } = useFormState({ control })
  const vehicleTypeOptions = useVehicleTypeOptions()
  const vehicleAvailabilityOptions = useVehicleAvailabilityOptions()
  const drivingLicenseOptions = useDrivingLicenseOptions()
  const { manifests } = useActiveModules()

  const planningKinds = new Set(manifests.map((m) => m.planning?.capacityKind))
  const plannedKinds = FORM_CAPACITY_KINDS.filter((kind) => planningKinds.has(kind))
  // Without a planning module every kind stays editable, so a vehicle never becomes locked.
  const visibleKinds = plannedKinds.length > 0 ? plannedKinds : FORM_CAPACITY_KINDS
  const planningModules = (kind: CapacityKind) =>
    manifests
      .filter((m) => m.planning?.capacityKind === kind)
      .map((m) => i18n.t(m.planning!.labelKey))
  const capacitiesEmpty = errors.waterCapacity?.type === EMPTY_CAPACITIES_KEY

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-y-6 lg:gap-x-11"
      onSubmit={handleSubmit(props.onSubmit)}
      // The domain validator owns every message; native bubbles would compete with it.
      noValidate
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
      <fieldset
        className="flex flex-col gap-y-4 lg:col-span-2"
        aria-describedby="vehicle-capacities-hint"
      >
        <legend className="mb-1 font-lato text-base font-semibold text-dark">
          {t('form.capacity.legend')}
        </legend>
        <p id="vehicle-capacities-hint" className="text-sm text-muted-foreground">
          {t('form.capacity.hint')}
        </p>
        {capacitiesEmpty && (
          <p role="alert" aria-live="assertive" className="text-sm text-destructive">
            {errors.waterCapacity?.message}
          </p>
        )}
        <div className="flex flex-col gap-6 lg:grid lg:grid-cols-2 lg:gap-x-11">
          {visibleKinds.map((kind) => (
            <CapacityField key={kind} kind={kind} modules={planningModules(kind)} />
          ))}
        </div>
      </fieldset>
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

      <FormSubmitButton />
    </form>
  )
}

export default FormForVehicle
