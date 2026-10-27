import { capacityDisplay } from '@/lib/capacity'
import { Suspense, useEffect } from 'react'
import {
  DatePickerField,
  TextareaField,
  SelectField,
  Label,
  MultiSelect,
  Checkbox,
} from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import type { User, Vehicle } from '@/api/backendApi'
import type { DrivingLicense } from '@green-ecolution/backend-client'
import { useDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { validateDriverLicenses } from '@/lib/licenseValidation'
import { Controller, SubmitHandler, useFormContext, useFormState, useWatch } from 'react-hook-form'
import { useQuery } from '@tanstack/react-query'
import { routingStartPointsQuery } from '@/api/queries'
import { planningManifests } from '@/modules/planning'
import type { PlanFormContext } from '@/modules/types'
import { useActiveModules } from '@/modules/useActiveModules'

interface FormForWateringPlanProps {
  displayError: boolean
  errorMessage?: string
  transporters: Vehicle[]
  trailers: Vehicle[]
  users: User[]
  formType: PlanFormContext['type']
  planId?: string
  organizationId?: string
  saveDraft: () => void
  onSubmit: SubmitHandler<WateringPlanForm>
  onBlur?: () => void
}

const startOfToday = new Date()
startOfToday.setHours(0, 0, 0, 0)

const FormForWateringPlan = (props: FormForWateringPlanProps) => {
  const { t, i18n } = useTranslation(['wateringPlan', 'common'])
  const { register, handleSubmit, control, resetField, getValues, setValue } =
    useFormContext<WateringPlanForm>()
  const { errors } = useFormState({ control })
  const { manifests } = useActiveModules()
  const planningModules = planningManifests(manifests)

  const { data: startPoints, isPending } = useQuery(routingStartPointsQuery())
  const getDrivingLicenseDetails = useDrivingLicenseDetails()

  useEffect(() => {
    if (!startPoints?.length) return
    if (!getValues('startPointName')) {
      const defaultPoint = startPoints.find((sp) => sp.isDefault) ?? startPoints[0]
      // Untouched select must submit the default depot, not undefined. resetField
      // moves the field's defaultValue along with the value, so the preselection
      // does not leave the form permanently diverged from its defaults — setValue
      // would make it count as dirty for the rest of its life.
      resetField('startPointName', { defaultValue: defaultPoint.name })
    }
  }, [startPoints, getValues, resetField])

  const watchedTransporterId = useWatch<WateringPlanForm, 'transporterId'>({
    name: 'transporterId',
  })
  const watchedTrailerId = useWatch<WateringPlanForm, 'trailerId'>({ name: 'trailerId' })
  const watchedDriverIds = useWatch<WateringPlanForm, 'driverIds'>({ name: 'driverIds' })
  const watchedModule = useWatch<WateringPlanForm, 'module'>({ name: 'module' })
  const planManifest = planningModules.find(({ key }) => key === watchedModule)
  const StopPicker = planManifest?.planning.StopPicker
  const capacity = capacityDisplay[planManifest?.planning.capacityKind ?? 'water']
  // An existing plan never changes its module, and with one planning module there is nothing to pick.
  const choosesModule = props.formType === 'create' && planningModules.length > 1

  // Must resolve already-assigned drivers even if no longer selectable, or a
  // stored assignment would wrongly report a missing licence.
  const licenseCheck = validateDriverLicenses(
    watchedDriverIds ?? [],
    props.users,
    props.transporters,
    props.trailers,
    watchedTransporterId,
    watchedTrailerId,
    t,
  )

  const getDrivingLicensesString = (user: User) => {
    if (!user.drivingLicenses || user.drivingLicenses.length === 0) {
      return t('form.noDrivingLicenseLabel')
    }

    return user.drivingLicenses
      .map((drivingLicense: DrivingLicense) => getDrivingLicenseDetails(drivingLicense).label)
      .join(', ')
  }

  const selectableUsers = props.users.filter((user) => user.wateringPlanSelectable)

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
      // The domain validator owns every message; native bubbles would compete with it.
      noValidate
      onBlur={props.onBlur}
    >
      <div className="flex flex-col gap-y-6">
        {choosesModule && (
          <Controller
            name="module"
            control={control}
            render={({ field }) => (
              <SelectField
                id="module"
                label={t('form.moduleLabel')}
                placeholder={t('form.modulePlaceholder')}
                required
                value={field.value}
                // Radix echoes a spurious onValueChange('') when the value is set
                // programmatically; '' is never a real module.
                onValueChange={(module) => {
                  if (!module || module === field.value) return
                  field.onChange(module)
                  // Stops belong to one module; a plan cannot keep another one's.
                  setValue('stopIds', [], { shouldDirty: true })
                }}
                options={planningModules.map(({ key, planning }) => ({
                  value: key,
                  label: i18n.t(planning.labelKey),
                }))}
              />
            )}
          />
        )}
        <Controller
          control={control}
          name="date"
          render={({ field: { value, onChange } }) => (
            <DatePickerField
              label={t('form.dateLabel')}
              error={errors.date?.message}
              required
              value={value ? new Date(value) : undefined}
              onChange={(date) => onChange(date)}
              fromDate={startOfToday}
            />
          )}
        />
        <Controller
          name="transporterId"
          control={control}
          render={({ field }) => (
            <SelectField
              id="transporterId"
              label={t('form.transporterLabel')}
              placeholder={t('form.transporterPlaceholder')}
              required
              value={field.value ?? ''}
              onValueChange={(val) => field.onChange(val)}
              error={errors.transporterId?.message}
              options={props.transporters.map((transporter) => ({
                value: transporter.id.toString(),
                label: `${transporter.numberPlate} · ${getDrivingLicenseDetails(transporter.drivingLicense).label}`,
              }))}
            />
          )}
        />
        {!isPending &&
          (startPoints?.length ? (
            <Controller
              name="startPointName"
              control={control}
              render={({ field }) => (
                <SelectField
                  id="startPointName"
                  label={t('form.startPointLabel')}
                  placeholder={t('form.startPointPlaceholder')}
                  required
                  value={field.value ?? ''}
                  // Radix echoes a spurious onValueChange('') from its hidden
                  // native form select when the value is set programmatically;
                  // '' is never a real choice here, so ignore it.
                  onValueChange={(val) => {
                    if (val) field.onChange(val)
                  }}
                  error={errors.startPointName?.message}
                  options={startPoints.map((sp) => ({
                    value: sp.name,
                    label: sp.name,
                  }))}
                />
              )}
            />
          ) : (
            <div className="flex flex-col gap-y-2">
              <Label>
                {t('form.startPointLabel')}
                <span className="text-destructive ml-1">*</span>
              </Label>
              <p className="text-sm text-destructive">{t('form.noStartPointsError')}</p>
            </div>
          ))}
        <Controller
          name="trailerId"
          control={control}
          render={({ field }) => (
            <SelectField
              id="trailerId"
              label={t('form.trailerLabel')}
              placeholder={t('form.trailerPlaceholder')}
              value={field.value ?? '-1'}
              onValueChange={(val) => {
                const withoutTrailer = val === '-1'
                field.onChange(withoutTrailer ? undefined : val)
                // Without a trailer the vehicle's tank is the only one; the backend rejects leaving it out.
                if (withoutTrailer) setValue('transporterTankUsed', true, { shouldDirty: true })
              }}
              error={errors.trailerId?.message}
              options={[
                { value: '-1', label: t('form.noTrailerOption') },
                ...props.trailers.map((trailer) => ({
                  value: trailer.id.toString(),
                  label: `${trailer.numberPlate} · ${getDrivingLicenseDetails(trailer.drivingLicense).label}`,
                })),
              ]}
            />
          )}
        />
        {watchedTrailerId && watchedTrailerId !== '-1' && (
          <Controller
            name="transporterTankUsed"
            control={control}
            render={({ field }) => (
              <div className="flex items-start gap-x-3">
                <Checkbox
                  id="transporterTankUsed"
                  className="mt-0.5"
                  checked={field.value ?? true}
                  onCheckedChange={(checked) => field.onChange(checked === true)}
                  aria-describedby="transporterTankUsedHint"
                />
                <div className="flex flex-col gap-y-1">
                  <Label htmlFor="transporterTankUsed">
                    {t(capacity.transporterUsedFormLabelKey)}
                  </Label>
                  <p id="transporterTankUsedHint" className="text-sm text-muted-foreground">
                    {t(capacity.transporterUsedHintKey)}
                  </p>
                </div>
              </div>
            )}
          />
        )}
        <Controller
          name="driverIds"
          control={control}
          render={({ field }) => (
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="driverIds">
                {t('form.driversLabel')}
                <span className="text-destructive ml-1">*</span>
              </Label>
              <p className="text-sm text-muted-foreground">{t('form.driversMultiSelectHint')}</p>
              <MultiSelect
                id="driverIds"
                value={field.value}
                onChange={field.onChange}
                options={selectableUsers.map((user) => ({
                  value: user.id,
                  label: `${user.firstName} ${user.lastName} · ${getDrivingLicensesString(user)}`,
                }))}
              />
              {errors.driverIds?.message && (
                <p role="alert" aria-live="assertive" className="text-sm text-destructive">
                  {errors.driverIds.message}
                </p>
              )}
              {!licenseCheck.valid && (
                <p role="alert" aria-live="assertive" className="text-sm text-destructive">
                  {licenseCheck.message}
                </p>
              )}
            </div>
          )}
        />
        <TextareaField
          placeholder={t('common:form.notesPlaceholder')}
          label={t('common:form.shortDescriptionLabel')}
          error={errors.description?.message}
          {...register('description')}
        />
      </div>

      {StopPicker && (
        <Controller
          control={control}
          name="stopIds"
          render={({ field: { value, onChange } }) => (
            <Suspense fallback={null}>
              <StopPicker
                value={value}
                onChange={onChange}
                organizationId={props.organizationId}
                error={errors.stopIds?.message}
                form={{
                  type: props.formType,
                  planId: props.planId,
                  transporterId: watchedTransporterId,
                  trailerId: watchedTrailerId,
                  saveDraft: props.saveDraft,
                }}
              />
            </Suspense>
          )}
        />
      )}

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton disabled={!licenseCheck.valid} />
    </form>
  )
}

export default FormForWateringPlan
