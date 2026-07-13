import { TreeForm } from '@/schema/treeSchema'
import { FormField, TextareaField, SelectField, Button } from '@green-ecolution/ui'
import { Sensor, TreeClusterInList } from '@/api/backendApi'
import { MapPin } from 'lucide-react'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import { Controller, SubmitHandler, useFormContext, useFormState } from 'react-hook-form'

interface FormForTreeProps {
  isReadonly: boolean
  treeClusters: TreeClusterInList[]
  sensors: Sensor[]
  displayError: boolean
  errorMessage?: string
  onChangeLocation?: () => void
  onSubmit: SubmitHandler<TreeForm>
  onBlur?: () => void
  hideLocation?: boolean
  fullWidth?: boolean
}

const FormForTree = (props: FormForTreeProps) => {
  const { register, handleSubmit, getValues, control } = useFormContext<TreeForm>()
  const { isValid, errors } = useFormState({ control })

  return (
    <form
      className={
        props.fullWidth
          ? 'flex flex-col gap-y-6'
          : 'flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11'
      }
      onSubmit={handleSubmit(props.onSubmit)}
      onBlur={props.onBlur}
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
              <SelectField
                id="treeClusterId"
                label="Bewässerungsgruppe"
                placeholder="Wählen Sie eine Bewässerungsgruppe aus"
                value={field.value ?? '-1'}
                onValueChange={(val) => field.onChange(val === '-1' ? null : val)}
                error={errors.treeClusterId?.message}
                options={[
                  { value: '-1', label: 'Keine Bewässerungsgruppe' },
                  ...props.treeClusters.map((cluster) => ({
                    value: cluster.id.toString(),
                    label: cluster.name,
                  })),
                ]}
              />
            )}
          />
        )}
        <Controller
          name="sensorId"
          control={control}
          render={({ field }) => (
            <SelectField
              id="sensorId"
              label="Verknüpfter Sensor"
              placeholder="Wählen Sie einen Sensor aus, sofern vorhanden"
              value={field.value ?? '-1'}
              onValueChange={(val) => field.onChange(val === '-1' ? null : val)}
              error={errors.sensorId?.message}
              options={[
                { value: '-1', label: 'Kein Sensor' },
                ...props.sensors.map((sensor) => ({
                  value: sensor.id.toString(),
                  label: `Sensor ${sensor.id}`,
                })),
              ]}
            />
          )}
        />
        <TextareaField
          placeholder="Hier ist Platz für Notizen"
          label="Kurze Beschreibung"
          error={errors.description?.message}
          {...register('description')}
        />
      </div>

      {!props.isReadonly && !props.hideLocation && (
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

          {props.onChangeLocation && (
            <Button
              type="button"
              variant="outline"
              onClick={props.onChangeLocation}
              className="mt-6"
            >
              Standort des Baumes anpassen
              <MapPin />
            </Button>
          )}
        </div>
      )}

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton
        disabled={!isValid}
        className={props.fullWidth ? 'mt-8 w-full' : undefined}
      />
    </form>
  )
}

export default FormForTree
