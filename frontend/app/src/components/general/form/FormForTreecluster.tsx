import { useState } from 'react'
import {
  Button,
  FormField,
  TextareaField,
  Label,
  Combobox,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@green-ecolution/ui'
import { Controller, SubmitHandler, useFormContext, useFormState } from 'react-hook-form'
import { SoilConditionOptions } from '@/hooks/details/useDetailsForSoilCondition'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import FormError from './FormError'
import FormSubmitButton from './FormSubmitButton'
import SelectEntities from './types/SelectEntities'
import SoilTextureDialog from './soilTexture/SoilTextureDialog'
import SoilTriangleIcon from './soilTexture/SoilTriangleIcon'

interface FormForTreeClusterProps {
  onAddTrees?: () => void
  onSubmit: SubmitHandler<TreeclusterForm>
  displayError: boolean
  errorMessage?: string
  onBlur?: () => void
  fullWidth?: boolean
  emptyHint?: string
}

const FormForTreecluster = (props: FormForTreeClusterProps) => {
  const { handleSubmit, register, control } = useFormContext<TreeclusterForm>()
  const { isValid, errors } = useFormState({ control })
  const [soilDialogOpen, setSoilDialogOpen] = useState(false)

  return (
    <form
      key="cluster-register"
      className={
        props.fullWidth
          ? 'flex min-h-0 flex-1 flex-col gap-y-6'
          : 'flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11'
      }
      onSubmit={handleSubmit(props.onSubmit)}
      onBlur={props.onBlur}
    >
      <div className={props.fullWidth ? 'flex shrink-0 flex-col gap-y-6' : 'flex flex-col gap-y-6'}>
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
              <div className="flex gap-x-2">
                <div className="min-w-0 flex-1">
                  <Combobox
                    id="soilCondition"
                    options={SoilConditionOptions}
                    value={field.value}
                    onChange={field.onChange}
                    placeholder="Wähle eine Bodenbeschaffenheit aus"
                    searchPlaceholder="Code oder Bezeichnung suchen…"
                  />
                </div>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        aria-label="Bodenbeschaffenheit bestimmen"
                        onClick={() => setSoilDialogOpen(true)}
                      >
                        <SoilTriangleIcon />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Bodenbeschaffenheit bestimmen</TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>
              {errors.soilCondition?.message && (
                <p className="text-sm text-destructive">{errors.soilCondition.message}</p>
              )}
              <SoilTextureDialog
                open={soilDialogOpen}
                onOpenChange={setSoilDialogOpen}
                initialCondition={field.value}
                onApply={field.onChange}
              />
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
            onChange={onChange}
            entityIds={value}
            onAdd={props.onAddTrees}
            type="tree"
            label="Bäume"
            fill={props.fullWidth}
            emptyHint={props.emptyHint}
          />
        )}
      />

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton
        disabled={!isValid}
        className={props.fullWidth ? 'mt-2 w-full shrink-0' : undefined}
      />
    </form>
  )
}

export default FormForTreecluster
