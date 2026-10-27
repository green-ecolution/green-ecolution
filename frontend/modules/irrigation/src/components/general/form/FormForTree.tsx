import { TreeForm } from '../../../schema/treeSchema'
import { expandShortYear } from '../../../lib/plantingYear'
import {
  irrigationPlantingYearIsFuture,
  irrigationPlantingYearMax,
  irrigationPlantingYearMin,
} from '@green-ecolution/domain-wasm'
import { FormField, TextareaField, SelectField, Button } from '@green-ecolution/ui'
import { Sensor } from '@/api/backendApi'
import { TreeClusterInList } from '../../../api/client'
import { MapPin } from 'lucide-react'
import type { FocusEvent } from 'react'
import { useTranslation } from 'react-i18next'
import FormError from '@/components/general/form/FormError'
import FormSubmitButton from '@/components/general/form/FormSubmitButton'
import { Controller, SubmitHandler, useFormContext, useFormState, useWatch } from 'react-hook-form'

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
  const { t } = useTranslation(['tree', 'common'])
  const { register, handleSubmit, getValues, control, setValue } = useFormContext<TreeForm>()
  const { errors } = useFormState({ control })

  const plantingYearField = register('plantingYear', { valueAsNumber: true })

  // useWatch, not watch: under the React Compiler `watch` does not re-render
  // reliably (see the frontend notes in CLAUDE.md).
  const watchedPlantingYear = useWatch<TreeForm, 'plantingYear'>({
    control,
    name: 'plantingYear',
  })
  const isPlannedPlanting =
    Number.isInteger(watchedPlantingYear) && irrigationPlantingYearIsFuture(watchedPlantingYear)

  const handlePlantingYearBlur = (event: FocusEvent<HTMLInputElement>) => {
    void plantingYearField.onBlur(event)

    const entered = event.target.valueAsNumber
    if (Number.isNaN(entered)) return

    const expanded = expandShortYear(entered)
    if (expanded !== entered) {
      setValue('plantingYear', expanded, { shouldValidate: true, shouldDirty: true })
    }
  }

  return (
    <form
      className={
        props.fullWidth
          ? 'flex shrink-0 flex-col gap-y-6'
          : 'flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-11'
      }
      onSubmit={handleSubmit(props.onSubmit)}
      // The domain validator owns every message; native bubbles would compete with it.
      noValidate
      onBlur={props.onBlur}
    >
      <div className="flex flex-col gap-y-6">
        {!props.isReadonly && (
          <FormField
            placeholder={t('form.numberLabel')}
            label={t('form.numberLabel')}
            required
            error={errors.number?.message}
            {...register('number')}
          />
        )}
        {!props.isReadonly && (
          <FormField
            placeholder={t('form.speciesLabel')}
            label={t('form.speciesLabel')}
            required
            error={errors.species?.message}
            {...register('species')}
          />
        )}
        {!props.isReadonly && (
          <FormField
            placeholder={t('form.plantingYearLabel')}
            label={t('form.plantingYearLabel')}
            type="number"
            min={irrigationPlantingYearMin()}
            max={irrigationPlantingYearMax()}
            error={errors.plantingYear?.message}
            description={isPlannedPlanting ? t('form.plantingYearFutureHint') : undefined}
            required
            {...plantingYearField}
            onBlur={handlePlantingYearBlur}
          />
        )}
        {!props.isReadonly && (
          <Controller
            name="treeClusterId"
            control={control}
            render={({ field }) => (
              <SelectField
                id="treeClusterId"
                label={t('form.clusterLabel')}
                placeholder={t('form.clusterPlaceholder')}
                value={field.value ?? '-1'}
                onValueChange={(val) => field.onChange(val === '-1' ? null : val)}
                error={errors.treeClusterId?.message}
                options={[
                  { value: '-1', label: t('form.clusterNoneOption') },
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
              label={t('form.sensorLabel')}
              placeholder={t('form.sensorPlaceholder')}
              value={field.value ?? '-1'}
              onValueChange={(val) => field.onChange(val === '-1' ? null : val)}
              error={errors.sensorId?.message}
              // The backend rejects this pairing; saying so beats a control
              // that is greyed out without explanation.
              disabled={isPlannedPlanting}
              description={isPlannedPlanting ? t('form.sensorFutureDisabledHint') : undefined}
              options={[
                { value: '-1', label: t('form.sensorNoneOption') },
                ...props.sensors.map((sensor) => ({
                  value: sensor.id.toString(),
                  label: t('form.sensorOptionLabel', { id: sensor.id }),
                })),
              ]}
            />
          )}
        />
        <TextareaField
          placeholder={t('common:form.notesPlaceholder')}
          label={t('common:form.shortDescriptionLabel')}
          error={errors.description?.message}
          {...register('description')}
        />
      </div>

      {!props.isReadonly && !props.hideLocation && (
        <div>
          <p className="block font-semibold text-dark-800 mb-2.5">{t('form.locationHeading')}</p>
          <div>
            <p className="block mb-2.5">
              <strong className="text-dark-800">{t('form.latitudeLabel')}</strong>{' '}
              {getValues('latitude')}
            </p>
            <p className="block mb-2.5">
              <strong className="text-dark-800 font-semibold">{t('form.longitudeLabel')}</strong>{' '}
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
              {t('form.adjustLocationButton')}
              <MapPin />
            </Button>
          )}
        </div>
      )}

      <FormError show={props.displayError} error={props.errorMessage} />

      <FormSubmitButton className={props.fullWidth ? 'mt-8 w-full' : undefined} />
    </form>
  )
}

export default FormForTree
