import PrimaryButton from '../buttons/PrimaryButton'
import Input from './types/Input'
import Select from './types/Select'
import Textarea from './types/Textarea'
import { Controller, SubmitHandler, useFormContext } from 'react-hook-form'
import { SoilConditionOptions } from '@/hooks/details/useDetailsForSoilCondition'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import FormError from './FormError'
import SelectEntities from './types/SelectEntities'

interface FormForTreeClusterProps {
  onAddTrees: () => void
  onSubmit: SubmitHandler<TreeclusterForm>
  displayError: boolean
  errorMessage?: string
}

const FormForTreecluster = (props: FormForTreeClusterProps) => {
  const {
    handleSubmit,
    register,
    formState: { isValid, errors },
    control,
  } = useFormContext<TreeclusterForm>()

  return (
    <form
      key="cluster-register"
      className="space-y-6 lg:space-y-0 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <div className="space-y-6">
        <Input label="Name" error={errors.name?.message} required {...register('name')} />
        <Input label="Adresse" required error={errors.address?.message} {...register('address')} />
        <Select
          options={SoilConditionOptions}
          placeholder="Wählen Sie eine Bodenbeschaffenheit aus"
          label="Bodenbeschaffenheit"
          required
          error={errors.soilCondition?.message}
          {...register('soilCondition')}
        />
        <Textarea
          placeholder="Hier ist Platz für Notizen"
          label="Kurze Beschreibung"
          error={errors.description?.message}
          {...register('description')}
        />
      </div>

      <Controller
        control={control}
        name="treeIds"
        render={({ field: { value, onChange } }) => (
          <SelectEntities
            // eslint-disable-next-line
            onDelete={() => {}} // TODO: remove
            onChange={onChange}
            entityIds={value}
            onAdd={props.onAddTrees}
            type="tree"
            label="Bäume"
          />
        )}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <PrimaryButton
        type="submit"
        label="Speichern"
        disabled={!isValid}
        className="mt-10 lg:col-span-full lg:w-fit"
      />
    </form>
  )
}

export default FormForTreecluster
