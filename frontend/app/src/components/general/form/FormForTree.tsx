import { TreeForm } from '@/schema/treeSchema'
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
import { Sensor, TreeCluster } from '@/api/backendApi'
import { MapPin, MoveRight } from 'lucide-react'
import FormError from './FormError'
import { Controller, SubmitHandler, useFormContext } from 'react-hook-form'

interface FormForTreeProps {
  isReadonly: boolean
  treeClusters: TreeCluster[]
  sensors: Sensor[]
  displayError: boolean
  errorMessage?: string
  onChangeLocation: () => void
  onSubmit: SubmitHandler<TreeForm>
}

const FormForTree = (props: FormForTreeProps) => {
  const {
    register,
    handleSubmit,
    getValues,
    control,
    formState: { isValid, errors },
  } = useFormContext<TreeForm>()

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <div className="flex flex-col gap-y-6">
        {!props.isReadonly && (
          <FormField
            placeholder="Baumnummer"
            label="Baumnummer"
            required
            error={errors.number?.message}
            {...register('number')}
          />
        )}
        {!props.isReadonly && (
          <FormField
            placeholder="Baumart"
            label="Baumart"
            required
            error={errors.species?.message}
            {...register('species')}
          />
        )}
        {!props.isReadonly && (
          <FormField
            placeholder="Pflanzjahr"
            label="Pflanzjahr"
            type="number"
            error={errors.plantingYear?.message}
            required
            {...register('plantingYear', { valueAsNumber: true })}
          />
        )}
        {!props.isReadonly && (
          <Controller
            name="treeClusterId"
            control={control}
            render={({ field }) => (
              <div className="flex flex-col gap-y-2">
                <Label htmlFor="treeClusterId">Bewässerungsgruppe</Label>
                <Select
                  value={field.value?.toString() ?? '-1'}
                  onValueChange={(val) => field.onChange(Number(val))}
                >
                  <SelectTrigger id="treeClusterId">
                    <SelectValue placeholder="Wählen Sie eine Bewässerungsgruppe aus" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="-1">Keine Bewässerungsgruppe</SelectItem>
                    {props.treeClusters.map((cluster) => (
                      <SelectItem key={cluster.id} value={cluster.id.toString()}>
                        {cluster.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {errors.treeClusterId?.message && (
                  <p className="text-sm text-destructive">{errors.treeClusterId.message}</p>
                )}
              </div>
            )}
          />
        )}
        <Controller
          name="sensorId"
          control={control}
          render={({ field }) => (
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="sensorId">Verknüpfter Sensor</Label>
              <Select value={field.value ?? '-1'} onValueChange={field.onChange}>
                <SelectTrigger id="sensorId">
                  <SelectValue placeholder="Wählen Sie einen Sensor aus, sofern vorhanden" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="-1">Kein Sensor</SelectItem>
                  {props.sensors.map((sensor) => (
                    <SelectItem key={sensor.id} value={sensor.id.toString()}>
                      Sensor {sensor.id}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.sensorId?.message && (
                <p className="text-sm text-destructive">{errors.sensorId.message}</p>
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

      {!props.isReadonly && (
        <div>
          <p className="block font-semibold text-dark-800 mb-2.5">Standort des Baumes</p>
          <div>
            <p className="block mb-2.5">
              <strong className="text-dark-800">Breitengrad:</strong> {getValues('latitude')}
            </p>
            <p className="block mb-2.5">
              <strong className="text-dark-800 font-semibold">Längengrad:</strong>{' '}
              {getValues('longitude')}
            </p>
          </div>

          <Button type="button" variant="outline" onClick={props.onChangeLocation} className="mt-6">
            Standort des Baumes anpassen
            <MapPin />
          </Button>
        </div>
      )}

      <FormError show={props.displayError} error={props.errorMessage} />

      <Button type="submit" className="mt-10 lg:col-span-full lg:w-fit" disabled={!isValid}>
        Speichern
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default FormForTree
