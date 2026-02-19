import { MoveRight } from 'lucide-react'
import {
  DatePickerField,
  TextareaField,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  MultiSelect,
  Button,
} from '@green-ecolution/ui'
import FormError from './FormError'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { User, Vehicle } from '@green-ecolution/backend-client'
import SelectEntities from './types/SelectEntities'
import { getDrivingLicenseDetails } from '@/hooks/details/useDetailsForDrivingLicense'
import { validateDriverLicenses } from '@/hooks/details/useLicenseValidation'
import { Controller, SubmitHandler, useFormContext, useWatch } from 'react-hook-form'

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

const FormForWateringPlan = (props: FormForWateringPlanProps) => {
  const {
    register,
    handleSubmit,
    control,
    formState: { isValid, errors },
  } = useFormContext<WateringPlanForm>()

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
      .map((drivingLicense) => getDrivingLicenseDetails(drivingLicense).label)
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
            />
          )}
        />
        <Controller
          name="transporterId"
          control={control}
          render={({ field }) => (
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="transporterId">
                Verknüpftes Fahrzeug
                <span className="text-destructive ml-1">*</span>
              </Label>
              <Select
                value={field.value?.toString()}
                onValueChange={(val) => field.onChange(Number(val))}
              >
                <SelectTrigger id="transporterId">
                  <SelectValue placeholder="Wählen Sie ein Fahrzeug aus" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="-1">Kein Fahrzeug</SelectItem>
                  {props.transporters.map((transporter) => (
                    <SelectItem key={transporter.id} value={transporter.id.toString()}>
                      {transporter.numberPlate} ·{' '}
                      {getDrivingLicenseDetails(transporter.drivingLicense).label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.transporterId?.message && (
                <p className="text-sm text-destructive">{errors.transporterId.message}</p>
              )}
            </div>
          )}
        />
        <Controller
          name="trailerId"
          control={control}
          render={({ field }) => (
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="trailerId">Verknüpfter Anhänger</Label>
              <Select
                value={field.value?.toString() ?? '-1'}
                onValueChange={(val) => field.onChange(val === '-1' ? undefined : Number(val))}
              >
                <SelectTrigger id="trailerId">
                  <SelectValue placeholder="Wählen Sie einen Anhänger aus, sofern vorhanden" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="-1">Keinen Anhänger</SelectItem>
                  {props.trailers.map((trailer) => (
                    <SelectItem key={trailer.id} value={trailer.id.toString()}>
                      {trailer.numberPlate} ·{' '}
                      {getDrivingLicenseDetails(trailer.drivingLicense).label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.trailerId?.message && (
                <p className="text-sm text-destructive">{errors.trailerId.message}</p>
              )}
            </div>
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
            // eslint-disable-next-line
            onDelete={() => {}} // TODO: remove
            onChange={onChange}
            entityIds={value}
            onAdd={props.onAddCluster}
            type="cluster"
            label="Bewässerungsgruppen"
          />
        )}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <Button
        type="submit"
        disabled={!isValid || !licenseCheck.valid}
        className="mt-10 lg:col-span-full lg:w-fit"
      >
        Speichern
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default FormForWateringPlan
