import { TreeForm } from '@/schema/treeSchema'
import Input from './types/Input'
import Select from './types/Select'
import { Sensor, TreeCluster } from '@/api/backendApi'
import Textarea from './types/Textarea'
import { MapPin } from 'lucide-react'
import PrimaryButton from '../buttons/PrimaryButton'
import FormError from './FormError'
import { SubmitHandler, useFormContext } from 'react-hook-form'

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
    formState: { isValid, errors },
  } = useFormContext<TreeForm>()

  return (
    <form
      className="space-y-6 lg:space-y-0 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <div className="space-y-6">
        {!props.isReadonly && (
          <Input
            placeholder="Baumnummer"
            label="Baumnummer"
            required
            error={errors.number?.message}
            {...register('number')}
          />
        )}
        {!props.isReadonly && (
          <Input
            placeholder="Baumart"
            label="Baumart"
            required
            error={errors.species?.message}
            {...register('species')}
          />
        )}
        {!props.isReadonly && (
          <Input
            placeholder="Pflanzjahr"
            label="Pflanzjahr"
            type="number"
            error={errors.plantingYear?.message}
            required
            {...register('plantingYear', { valueAsNumber: true })}
          />
        )}
        {!props.isReadonly && (
          <Select
            options={[
              { label: 'Keine Bewässerungsgruppe', value: '-1' },
              ...props.treeClusters.map((cluster) => ({
                label: cluster.name,
                value: cluster.id.toString(),
              })),
            ]}
            placeholder="Wählen Sie eine Bewässerungsgruppe aus"
            label="Bewässerungsgruppe"
            error={errors.treeClusterId?.message}
            {...register('treeClusterId')}
          />
        )}
        <Select
          options={[
            { label: 'Kein Sensor', value: '-1' },
            ...props.sensors.map((sensor) => ({
              label: `Sensor ${sensor.id.toString()}`,
              value: sensor.id.toString(),
            })),
          ]}
          placeholder="Wählen Sie einen Sensor aus, sofern vorhanden"
          label="Verknüpfter Sensor"
          error={errors.sensorId?.message}
          {...register('sensorId')}
        />
        <Textarea
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

          <button
            type="button"
            className="mt-6 w-fit border border-green-light text-dark-800 px-5 py-2 group flex gap-x-3 rounded-xl items-center transition-all ease-in-out duration-300 hover:border-green-dark hover:text-dark"
            onClick={props.onChangeLocation}
          >
            <span className="font-medium">Standort des Baumes anpassen</span>
            <MapPin className="text-current" />
          </button>
        </div>
      )}

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

export default FormForTree
