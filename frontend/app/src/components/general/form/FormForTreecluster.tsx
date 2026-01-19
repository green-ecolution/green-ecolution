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
  Button,
} from '@green-ecolution/ui'
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
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11"
      onSubmit={handleSubmit(props.onSubmit)}
    >
      <div className="flex flex-col gap-y-6">
        <FormField label="Name" error={errors.name?.message} required {...register('name')} />
        <FormField
          label="Adresse"
          required
          error={errors.address?.message}
          {...register('address')}
        />
        <Controller
          name="soilCondition"
          control={control}
          render={({ field }) => (
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="soilCondition">
                Bodenbeschaffenheit
                <span className="text-destructive ml-1">*</span>
              </Label>
              <Select value={field.value} onValueChange={field.onChange}>
                <SelectTrigger id="soilCondition">
                  <SelectValue placeholder="Wählen Sie eine Bodenbeschaffenheit aus" />
                </SelectTrigger>
                <SelectContent>
                  {SoilConditionOptions.map((opt) => (
                    <SelectItem key={opt.value} value={opt.value}>
                      {opt.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {errors.soilCondition?.message && (
                <p className="text-sm text-destructive">{errors.soilCondition.message}</p>
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

      <Button type="submit" disabled={!isValid} className="mt-10 lg:col-span-full lg:w-fit">
        Speichern
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default FormForTreecluster
