import { useCallback, useState } from 'react'
import FormPageHeader from '../general/FormPageHeader'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan, WateringPlanUpdate } from '@/api/backendApi'
import { wateringPlanIdQuery } from '@/api/queries'
import { format } from 'date-fns'
import { Droplet, MoveRight } from 'lucide-react'
import FormError from '../general/form/FormError'
import {
  getWateringPlanStatusDetails,
  WateringPlanStatusOptions,
} from '@/hooks/details/useDetailsForWateringPlanStatus'
import { Badge, TextareaField, FormField, SelectField, Button } from '@green-ecolution/ui'
import {
  WateringPlanFinishedForm,
  wateringPlanFinishedSchema,
  WateringPlanCancelForm,
  wateringPlanCancelSchema,
} from '@/schema/wateringPlanSchema'
import { zodResolver } from '@/lib/zodResolver'
import { SubmitHandler, useFieldArray, useForm } from 'react-hook-form'
import { useMutation, useQueryClient, useSuspenseQuery } from '@tanstack/react-query'
import { wateringPlanApi } from '@/api/backendApi'
import { useNavigate } from '@tanstack/react-router'
import createToast from '@/hooks/createToast'

interface WateringPlanStatusUpdateProps {
  wateringPlanId: string
}

const WateringPlanStatusUpdate = ({ wateringPlanId }: WateringPlanStatusUpdateProps) => {
  const { data: loadedData } = useSuspenseQuery(wateringPlanIdQuery(wateringPlanId))
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const showToast = createToast()
  const statusDetails = getWateringPlanStatusDetails(loadedData.status)
  const [selectedStatus, setSelectedStatus] = useState(statusDetails)

  const { mutate, isError, error } = useMutation({
    mutationFn: (wateringPlan: WateringPlanUpdate) =>
      wateringPlanApi.updateWateringPlan({
        wateringPlanId: wateringPlanId,
        wateringPlanUpdateRequest: wateringPlan,
      }),

    onSuccess: (data: WateringPlan) => {
      queryClient
        .invalidateQueries(wateringPlanIdQuery(String(data.id)))
        .catch((error) => console.error('Invalidate "waterinPlanIdQuery" failed', error))
      queryClient
        .invalidateQueries({ queryKey: ['watering-plans'] })
        .catch((error) => console.error('Invalidate "watering-plans" failed:', error))

      navigate({
        to: `/watering-plans/$wateringPlanId`,
        params: { wateringPlanId: data.id.toString() },
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))

      showToast('Der Status der Einsatzplanung wurde erfolgreich aktualisiert.')
    },

    onError: (error) => {
      console.error('Error with vehicle mutation:', error)
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error') // TODO: Parse API ResponseError
    },
    throwOnError: true,
  })

  const date = format(new Date(loadedData.date), 'dd.MM.yyyy')

  const formByStatus = useCallback(
    (status: WateringPlanStatus) => {
      const onSubmitFinished: SubmitHandler<WateringPlanFinishedForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.Finished,
          evaluation: data.evaluation,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }

      const onSubmitCancel: SubmitHandler<WateringPlanCancelForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.Canceled,
          cancellationNote: data.cancellationNote,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }

      const onSubmitOtherStatus = (status: WateringPlanStatus) => {
        mutate({
          ...loadedData,
          status,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }
      switch (status) {
        case 'canceled':
          return <CancelWateringPlan onSubmit={onSubmitCancel} />
        case 'finished':
          return (
            <FinishedWateringPlan
              onSubmit={onSubmitFinished}
              wateringPlanId={wateringPlanId}
              loadedData={loadedData}
            />
          )
        default:
          return (
            <Button onClick={() => onSubmitOtherStatus(status)} type="submit" className="mt-10">
              Speichern
              <MoveRight className="icon-arrow-animate" />
            </Button>
          )
      }
    },
    [loadedData, wateringPlanId, mutate],
  )

  return (
    <>
      <FormPageHeader
        backLink={{
          label: 'Zurück zm Einsatzplan',
          link: {
            to: `/watering-plans/$wateringPlanId`,
            params: { wateringPlanId },
          },
        }}
        title={<>Status vom Einsatzplan {date} bearbeiten</>}
      >
        <p className="flex gap-x-3 mb-5">
          <strong>Aktueller Status:</strong>
          <Badge variant={statusDetails.color} size="lg">
            {statusDetails.label}
          </Badge>
        </p>
        <p>
          Der Status eines Einsatzes beschreibt, ob der Einsatz beispielsweise aktiv ausgeführt
          wird, beendet ist oder abgebrochen wurde. Diese Angabe hilft dabei die erstellen Einsätze
          zu kategorisieren und eine Auswertung anzulegen. Sobald ein Einsatz beendet wird, kann
          zudem angegeben werden, mit wie viel Wasser die zugehörigen Bewässerungsgruppen bewässert
          wurden.
        </p>
      </FormPageHeader>

      <section className="mt-10">
        <div className="flex flex-col gap-y-6 md:w-1/2">
          <SelectField
            id="status"
            label="Status des Einsatzes"
            placeholder="Wählen Sie einen Status aus"
            required
            value={selectedStatus.value}
            onValueChange={(value) => {
              setSelectedStatus(getWateringPlanStatusDetails(value))
            }}
            options={WateringPlanStatusOptions}
          />
        </div>
        {formByStatus(selectedStatus.value)}
        <FormError show={isError} error={error?.message} />
      </section>
    </>
  )
}

interface CancelPlanProps {
  onSubmit: SubmitHandler<WateringPlanCancelForm>
  submitLabel?: string
  className?: string
}

export const CancelWateringPlan = ({
  onSubmit,
  submitLabel = 'Speichern',
  className = 'md:w-1/2',
}: CancelPlanProps) => {
  const {
    register,
    handleSubmit,
    formState: { isValid, errors },
  } = useForm({
    mode: 'onChange',
    resolver: zodResolver(wateringPlanCancelSchema),
  })

  return (
    <form className={className} onSubmit={handleSubmit(onSubmit)}>
      <TextareaField
        placeholder="Warum wurde der Einsatz abgebrochen?"
        label="Grund des Abbruchs"
        error={errors.cancellationNote?.message}
        required
        {...register('cancellationNote')}
      />

      <Button type="submit" disabled={!isValid} className="mt-10">
        {submitLabel}
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

interface FinishedPlanProps {
  onSubmit: SubmitHandler<WateringPlanFinishedForm>
  wateringPlanId: string
  loadedData: Pick<WateringPlan, 'treeclusters'>
  submitLabel?: string
}

export const FinishedWateringPlan = ({
  wateringPlanId,
  onSubmit,
  loadedData,
  submitLabel = 'Speichern',
}: FinishedPlanProps) => {
  const {
    register,
    handleSubmit,
    formState: { isValid },
    control,
  } = useForm({
    mode: 'onChange',
    resolver: zodResolver(wateringPlanFinishedSchema),
    defaultValues: {
      evaluation: loadedData.treeclusters.map((cluster: { treeIds?: string[]; id: string }) => ({
        consumedWater: (cluster.treeIds?.length ?? 1) * 80,
        treeClusterId: cluster.id,
        wateringPlanId: wateringPlanId,
      })),
    },
  })

  const { fields } = useFieldArray({
    control,
    name: 'evaluation',
  })

  return (
    <form className="flex flex-col gap-y-6" onSubmit={handleSubmit(onSubmit)}>
      <fieldset className="mt-6">
        <legend className="block font-semibold text-dark-800 mb-2.5">
          Wasservergabe pro Bewässerungsgruppe:
        </legend>
        <p className="-mt-2 text-sm text-dark-600 mb-2.5">
          Die Standardwerte ergeben sich aus 80 Litern pro Baum einer Bewässerungsgruppe.
        </p>
        <ul className="flex flex-col">
          {fields.map((field, index) => {
            const cluster = loadedData.treeclusters[index]
            const treeCount = cluster.treeIds.length
            return (
              <li
                key={field.treeClusterId}
                className="flex items-center justify-between gap-4 border-b border-dark-100 py-3 last:border-0"
              >
                <div className="flex min-w-0 items-center gap-2.5">
                  <span
                    aria-hidden
                    className="flex size-8 shrink-0 items-center justify-center rounded-full bg-green-light-100"
                  >
                    <Droplet className="size-4 text-green-dark" />
                  </span>
                  <div className="min-w-0">
                    <p className="truncate font-medium text-dark">{cluster.name}</p>
                    {treeCount > 0 && (
                      <p className="text-xs tabular-nums text-dark-600">
                        {treeCount} {treeCount === 1 ? 'Baum' : 'Bäume'}
                      </p>
                    )}
                  </div>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <FormField
                    type="number"
                    label={`Liter für ${cluster.name}`}
                    defaultValue={field.consumedWater}
                    className="max-w-28"
                    hideLabel
                    {...register(`evaluation.${index}.consumedWater`)}
                  />
                  <span className="text-sm text-dark-600">Liter</span>
                </div>
              </li>
            )
          })}
        </ul>
      </fieldset>

      <Button type="submit" disabled={!isValid} className="mt-10">
        {submitLabel}
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </form>
  )
}

export default WateringPlanStatusUpdate
