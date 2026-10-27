import { useMemo } from 'react'
import { Controller, useForm, useWatch } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FormField,
  Label,
  SelectField,
} from '@green-ecolution/ui'
import type { CapacityKindDto, Organization, StartPointResponse } from '@/api/backendApi'
import { capacityDisplay } from '@/lib/capacity'
import DraggableMarker from '@/components/map-gl/DraggableMarker'
import MapPreview from '@/components/map-gl/MapPreview'

const NAME_MAX_LENGTH = 255

const CAPACITY_KINDS = Object.keys(capacityDisplay) as CapacityKindDto[]

export interface StationPayload {
  name: string
  lat: number
  lon: number
  organizationId?: string
  replenishes: CapacityKindDto[]
}

interface StationDialogProps {
  open: boolean
  mode: 'create' | 'edit'
  organizations: Organization[]
  station?: StartPointResponse
  initialPosition?: { lat: number; lng: number }
  saving?: boolean
  onOpenChange: (open: boolean) => void
  onSubmit: (payload: StationPayload) => void
}

interface FormValues {
  name: string
  organizationId: string
  replenishes: CapacityKindDto[]
  lat: number | null
  lon: number | null
}

type FormProps = Omit<StationDialogProps, 'open' | 'onOpenChange'> & {
  onCancel: () => void
}

const StationForm = ({
  mode,
  organizations,
  station,
  initialPosition,
  saving = false,
  onSubmit,
  onCancel,
}: FormProps) => {
  const { t } = useTranslation(['settings', 'common'])

  const schema = useMemo(
    () =>
      z.object({
        name: z
          .string()
          .trim()
          .min(1, t('stations.dialog.nameRequired'))
          .max(NAME_MAX_LENGTH, t('stations.dialog.nameTooLong')),
        organizationId:
          mode === 'create'
            ? z.string().min(1, t('stations.dialog.organizationRequired'))
            : z.string(),
        replenishes: z.array(z.enum(['water', 'cargo_volume'])),
        lat: z
          .number()
          .nullable()
          .refine((v) => v !== null && v >= -90 && v <= 90, t('stations.dialog.positionRequired')),
        lon: z
          .number()
          .nullable()
          .refine(
            (v) => v !== null && v >= -180 && v <= 180,
            t('stations.dialog.positionRequired'),
          ),
      }),
    [mode, t],
  )

  const start = {
    lat: station?.lat ?? initialPosition?.lat ?? null,
    lng: station?.lon ?? initialPosition?.lng ?? null,
  }

  const { register, control, handleSubmit, setValue, formState } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      name: station?.name ?? '',
      organizationId: station?.organizationId ?? organizations[0]?.id ?? '',
      replenishes: station?.replenishes ?? [],
      lat: start.lat,
      lon: start.lng,
    },
  })

  const lat = useWatch({ control, name: 'lat' })
  const lon = useWatch({ control, name: 'lon' })
  const { errors } = formState

  const submit = handleSubmit((values) => {
    if (saving || values.lat === null || values.lon === null) return
    onSubmit({
      name: values.name.trim(),
      lat: values.lat,
      lon: values.lon,
      ...(mode === 'create' ? { organizationId: values.organizationId } : {}),
      replenishes: values.replenishes,
    })
  })

  return (
    <form onSubmit={submit} className="flex min-h-0 flex-col gap-4" noValidate>
      <FormField
        id="station-name"
        label={t('stations.dialog.nameLabel')}
        error={errors.name?.message}
        {...register('name')}
      />

      {mode === 'create' && (
        <Controller
          control={control}
          name="organizationId"
          render={({ field }) => (
            <SelectField
              id="station-organization"
              label={t('stations.dialog.organizationLabel')}
              placeholder={t('stations.dialog.organizationPlaceholder')}
              options={organizations.map((org) => ({ value: org.id, label: org.name }))}
              value={field.value}
              onValueChange={field.onChange}
              error={errors.organizationId?.message}
            />
          )}
        />
      )}

      <Controller
        control={control}
        name="replenishes"
        render={({ field }) => (
          <fieldset className="flex flex-col gap-2">
            <legend className="mb-1 text-sm font-medium">
              {t('stations.dialog.replenishesLegend')}
            </legend>
            {CAPACITY_KINDS.map((kind) => {
              const { icon: Icon } = capacityDisplay[kind]
              const id = `station-kind-${kind}`
              return (
                <div key={kind} className="flex items-center gap-2">
                  <Checkbox
                    id={id}
                    checked={field.value.includes(kind)}
                    onCheckedChange={(checked) =>
                      field.onChange(
                        checked ? [...field.value, kind] : field.value.filter((k) => k !== kind),
                      )
                    }
                  />
                  <Label htmlFor={id} className="flex items-center gap-2 font-normal">
                    <Icon className="size-4 text-dark-600" aria-hidden />
                    {t(`stations.dialog.replenishes.${kind}`)}
                  </Label>
                </div>
              )
            })}
            <p className="text-sm text-dark-600">{t('stations.dialog.replenishesHint')}</p>
          </fieldset>
        )}
      />

      <div className="flex flex-col gap-2">
        <span className="text-sm font-medium">{t('stations.dialog.positionLabel')}</span>
        <MapPreview
          interactive
          center={start.lng !== null && start.lat !== null ? [start.lng, start.lat] : undefined}
          zoom={15}
          ariaLabel={t('stations.dialog.mapAriaLabel')}
          className="h-56"
        >
          {lat !== null && lon !== null && (
            <DraggableMarker
              lat={lat}
              lng={lon}
              onDragEnd={({ lat: nextLat, lng }) => {
                setValue('lat', nextLat, { shouldDirty: true })
                setValue('lon', lng, { shouldDirty: true })
              }}
            />
          )}
        </MapPreview>
        {errors.lat || errors.lon ? (
          <p role="alert" className="text-sm text-destructive">
            {t('stations.dialog.positionRequired')}
          </p>
        ) : (
          <p className="text-sm text-dark-600">{t('stations.dialog.positionHint')}</p>
        )}
      </div>

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          {t('common:actions.cancel')}
        </Button>
        <Button type="submit" disabled={saving}>
          {t('common:actions.save')}
        </Button>
      </DialogFooter>
    </form>
  )
}

const StationDialog = ({ open, onOpenChange, ...formProps }: StationDialogProps) => {
  const { t } = useTranslation('settings')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[calc(100dvh-3rem)] max-w-xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {formProps.mode === 'create'
              ? t('stations.dialog.createTitle')
              : t('stations.dialog.editTitle')}
          </DialogTitle>
          <DialogDescription>{t('stations.dialog.description')}</DialogDescription>
        </DialogHeader>
        <StationForm {...formProps} onCancel={() => onOpenChange(false)} />
      </DialogContent>
    </Dialog>
  )
}

export default StationDialog
