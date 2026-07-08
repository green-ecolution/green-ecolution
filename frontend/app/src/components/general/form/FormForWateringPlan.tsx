import {
  DatePickerField,
  TextareaField,
  SelectField,
  Label,
  MultiSelect,
} from '@green-ecolution/ui'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import type { User, Vehicle } from '@/api/backendApi'
import type { DrivingLicense } from '@green-ecolution/backend-client'
import SelectEntities from './types/SelectEntities'
import { getDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { validateDriverLicenses } from '@/lib/licenseValidation'
import { Controller, SubmitHandler, useFormContext, useFormState, useWatch } from 'react-hook-form'
import { useQuery } from '@tanstack/react-query'
import { routingStartPointsQuery } from '@/api/queries'

interface FormForWateringPlanProps {
  displayError: boolean
  errorMessage?: string
  transporters: Vehicle[]
  trailers: Vehicle[]
  users: User[]
  onAddCluster: () => void
  onSubmit: SubmitHandler<WateringPlanForm>
  onBlur?: () => void
}

const startOfToday = new Date()
startOfToday.setHours(0, 0, 0, 0)

const FormForWateringPlan = (props: FormForWateringPlanProps) => {
  const { register, handleSubmit, control } = useFormContext<WateringPlanForm>()
  const { isValid, errors } = useFormState({ control })

  const { data: startPoints } = useQuery(routingStartPointsQuery())

  const watchedTransporterId = useWatch<WateringPlanForm, 'transporterId'>({
    name: 'transporterId',
  })
  const watchedTrailerId = useWatch<WateringPlanForm, 'trailerId'>({ name: 'trailerId' })
  const watchedDriverIds = useWatch<WateringPlanForm, 'driverIds'>({ name: 'driverIds' })

  const licenseCheck = validateDriverLicenses(
    watchedDriverIds ?? [],
    props.users,
    props.transporters,
    props.trailers,
    watchedTransporterId,
    watchedTrailerId,
  )

  const getDrivingLicensesString = (user: User) => {
    if (!user.drivingLicenses || user.drivingLicenses.length === 0) {
      return 'Keinen Führerschein'
    }

    return user.drivingLicenses
      .map((drivingLicense: DrivingLicense) => getDrivingLicenseDetails(drivingLicense).label)
      .join(', ')
  }

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
      onBlur={props.onBlur}
    >
      <div className="flex flex-col gap-y-6">
        <Controller
          control={control}
          name="date"
          render={({ field: { value, onChange } }) => (
            <DatePickerField
              label="Datum des Einsatzes"
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
              label="Verknüpftes Fahrzeug"
              placeholder="Wählen Sie ein Fahrzeug aus"
              required
              value={field.value ?? ''}
              onValueChange={(val) => field.onChange(val)}
              error={errors.transporterId?.message}
              options={[
                { value: '-1', label: 'Kein Fahrzeug' },
                ...props.transporters.map((transporter) => ({
                  value: transporter.id.toString(),
                  label: `${transporter.numberPlate} · ${getDrivingLicenseDetails(transporter.drivingLicense).label}`,
                })),
              ]}
            />
          )}
        />
        {startPoints != null && (
          <Controller
            name="startPointName"
            control={control}
            render={({ field }) => (
              <SelectField
                id="startPointName"
                label="Startpunkt"
                placeholder="Startpunkt auswählen"
                value={field.value ?? startPoints[0]?.name ?? ''}
                onValueChange={(val) => field.onChange(val)}
                error={errors.startPointName?.message}
                options={startPoints.map((sp) => ({
                  value: sp.name,
                  label: sp.name,
                }))}
              />
            )}
          />
        )}
        <Controller
          name="trailerId"
          control={control}
          render={({ field }) => (
            <SelectField
              id="trailerId"
              label="Verknüpfter Anhänger"
              placeholder="Wählen Sie einen Anhänger aus, sofern vorhanden"
              value={field.value ?? '-1'}
              onValueChange={(val) => field.onChange(val === '-1' ? undefined : val)}
              error={errors.trailerId?.message}
              options={[
                { value: '-1', label: 'Keinen Anhänger' },
                ...props.trailers.map((trailer) => ({
                  value: trailer.id.toString(),
                  label: `${trailer.numberPlate} · ${getDrivingLicenseDetails(trailer.drivingLicense).label}`,
                })),
              ]}
            />
          )}
        />
        <Controller
          name="driverIds"
          control={control}
          render={({ field }) => (
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="driverIds">
                Verknüpfte Mitarbeitende
                <span className="text-destructive ml-1">*</span>
              </Label>
              <p className="text-sm text-muted-foreground">
                Indem Sie die Taste »Shift« gedrückt halten, können Sie eine Mehrauswahl tätigen.
              </p>
              <MultiSelect
                id="driverIds"
                value={field.value}
                onChange={field.onChange}
                options={props.users.map((user) => ({
                  value: user.id,
                  label: `${user.firstName} ${user.lastName} · ${getDrivingLicensesString(user)}`,
                }))}
              />
              {errors.driverIds?.message && (
                <p className="text-sm text-destructive">{errors.driverIds.message}</p>
              )}
              {!licenseCheck.valid && (
                <p className="text-sm text-destructive">{licenseCheck.message}</p>
              )}
            </div>
          )}
        />
        <TextareaField
          placeholder="Hier ist Platz für Notizen"
          label="Kurze Beschreibung"
          error={errors.description?.message}
          {...register('description')}
        />
      </div>

      <Controller
        control={control}
        name="clusterIds"
        render={({ field: { value, onChange } }) => (
          <SelectEntities
            onChange={onChange}
            entityIds={value}
            onAdd={props.onAddCluster}
            type="cluster"
            label="Bewässerungsgruppen"
          />
        )}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton disabled={!isValid || !licenseCheck.valid} />
    </form>
  )
}

export default FormForWateringPlan
