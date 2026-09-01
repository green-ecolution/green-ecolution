import { useState } from 'react'
import { useTranslation } from 'react-i18next'
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
import type { OrganizationResponse } from '@green-ecolution/backend-client'
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
  /** Selectable owning organizations; the field stays hidden below two. */
  organizations?: OrganizationResponse[]
  onOrganizationChange?: (organizationId: string) => void
  /** Trees dropped by the last organization change. */
  discardedTreeCount?: number
}

const FormForTreecluster = (props: FormForTreeClusterProps) => {
  const { t } = useTranslation(['treecluster', 'common'])
  const { handleSubmit, register, control } = useFormContext<TreeclusterForm>()
  const { isValid, errors } = useFormState({ control })
  const [soilDialogOpen, setSoilDialogOpen] = useState(false)
  const organizations = props.organizations ?? []

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
        {organizations.length > 1 && (
          <Controller
            name="organizationId"
            control={control}
            render={({ field }) => (
              <div className="flex flex-col gap-y-2">
                <Label htmlFor="organizationId">
                  {t('form.organizationLabel')}
                  <span className="text-destructive ml-1">*</span>
                </Label>
                <Combobox
                  id="organizationId"
                  options={organizations.map((org) => ({ value: org.id, label: org.name }))}
                  value={field.value}
                  onChange={props.onOrganizationChange ?? field.onChange}
                  placeholder={t('form.organizationPlaceholder')}
                  searchPlaceholder={t('form.organizationSearchPlaceholder')}
                />
                <p className="text-sm text-dark-600">{t('form.organizationHint')}</p>
                {(props.discardedTreeCount ?? 0) > 0 && (
                  <p className="text-sm text-destructive">
                    {t('form.discardedTree', { count: props.discardedTreeCount })}
                  </p>
                )}
              </div>
            )}
          />
        )}
        <FormField
          label={t('form.nameLabel')}
          error={errors.name?.message}
          required
          {...register('name')}
        />
        <FormField
          label={t('form.addressLabel')}
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
                {t('form.soilConditionLabel')}
                <span className="text-destructive ml-1">*</span>
              </Label>
              <div className="flex gap-x-2">
                <div className="min-w-0 flex-1">
                  <Combobox
                    id="soilCondition"
                    options={SoilConditionOptions}
                    value={field.value}
                    onChange={field.onChange}
                    placeholder={t('form.soilConditionPlaceholder')}
                    searchPlaceholder={t('form.soilConditionSearchPlaceholder')}
                  />
                </div>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        aria-label={t('form.soilConditionDetermineAriaLabel')}
                        onClick={() => setSoilDialogOpen(true)}
                      >
                        <SoilTriangleIcon />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>{t('form.soilConditionDetermineAriaLabel')}</TooltipContent>
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
          placeholder={t('common:form.notesPlaceholder')}
          label={t('common:form.shortDescriptionLabel')}
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
            label={t('form.treesLabel')}
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
