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
  MultiSelect,
  Button,
} from '@green-ecolution/ui'
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
            <FormField
              label="Datum des Einsatzes"
              error={errors.date?.message}
              required
              type="date"
              onChange={onChange}
              value={new Date(value).toISOString().split('T')[0]}
            />
          )}
        />
        <Controller
          name="transporterId"
          control={control}
          render={({ field }) => (
            <div className="space-y-2">
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
            <div className="space-y-2">
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
            <div className="space-y-2">
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
            type="tree"
            label="Bäume"
          />
        )}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <Button type="submit" disabled={!isValid} className="mt-10 lg:col-span-full lg:w-fit">
        Speichern
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default FormForWateringPlan
