import {
  Controller,
  useFormContext,
  useFormState,
  useWatch,
  type SubmitHandler,
} from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Combobox, FormField, Label } from '@green-ecolution/ui'
import type { WasteBinForm } from '@green-ecolution/domain-wasm'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import FormError from '@/components/general/form/FormError'
import FormSubmitButton from '@/components/general/form/FormSubmitButton'
import { parseDecimalInput } from '@/lib/utils'
import WasteBinPositionPicker, { type WasteBinPosition } from './WasteBinPositionPicker'

interface FormForWasteBinProps {
  onSubmit: SubmitHandler<WasteBinForm>
  displayError: boolean
  errorMessage?: string
  /** Selectable owning organizations; the field stays hidden below two. */
  organizations?: OrganizationResponse[]
  onBlur?: () => void
}

// The domain validator does not coerce, so a raw input string would reach the API.
const asNumber = { setValueAs: parseDecimalInput } as const

const FormForWasteBin = ({
  onSubmit,
  displayError,
  errorMessage,
  organizations = [],
  onBlur,
}: FormForWasteBinProps) => {
  const { t } = useTranslation('waste')
  const { register, handleSubmit, control, setValue } = useFormContext<WasteBinForm>()
  const { errors } = useFormState({ control })
  const latitude = useWatch({ control, name: 'latitude' })
  const longitude = useWatch({ control, name: 'longitude' })

  const placeBin = ({ lat, lng }: WasteBinPosition) => {
    setValue('latitude', lat, { shouldValidate: true, shouldDirty: true })
    setValue('longitude', lng, { shouldValidate: true, shouldDirty: true })
  }

  const positionMissing = latitude == null || longitude == null
  const positionError = errors.latitude ?? errors.longitude
  const positionMessage = positionError
    ? positionMissing
      ? t('form.positionRequired')
      : positionError.message
    : undefined

  return (
    <form
      className="flex flex-col gap-y-6 lg:grid lg:grid-cols-2 lg:gap-x-11"
      onSubmit={handleSubmit(onSubmit)}
      onBlur={onBlur}
      // The domain validator owns every message; native bubbles would compete with it.
      noValidate
    >
      <div className="flex flex-col gap-y-6">
        <FormField
          label={t('form.numberLabel')}
          placeholder={t('form.numberPlaceholder')}
          required
          error={errors.number?.message}
          {...register('number')}
        />
        <FormField
          label={t('form.volumeLabel')}
          placeholder={t('form.volumePlaceholder')}
          type="number"
          required
          error={errors.volumeLiters?.message}
          {...register('volumeLiters', asNumber)}
        />
        <FormField
          label={t('form.depthLabel')}
          placeholder={t('form.depthPlaceholder')}
          type="number"
          step="1"
          required
          description={t('form.depthHint')}
          error={errors.depthCm?.message}
          {...register('depthCm', asNumber)}
        />
        {organizations.length > 1 && (
          <Controller
            name="organizationId"
            control={control}
            render={({ field }) => (
              <div className="flex flex-col gap-y-2">
                <Label htmlFor="organizationId">
                  {t('form.organizationLabel')}
                  <span className="ml-1 text-destructive">*</span>
                </Label>
                <Combobox
                  id="organizationId"
                  options={organizations.map((org) => ({ value: org.id, label: org.name }))}
                  value={field.value}
                  // Ignore the empty echo some pickers send on a programmatic change.
                  onChange={(value) => value && field.onChange(value)}
                  placeholder={t('form.organizationPlaceholder')}
                  searchPlaceholder={t('form.organizationSearchPlaceholder')}
                />
                <p className="text-sm text-dark-600">{t('form.organizationHint')}</p>
              </div>
            )}
          />
        )}
      </div>

      <WasteBinPositionPicker
        lat={latitude}
        lng={longitude}
        onChange={placeBin}
        error={positionMessage}
      />

      <FormError show={displayError} error={errorMessage} />
      <FormSubmitButton />
    </form>
  )
}

export default FormForWasteBin
