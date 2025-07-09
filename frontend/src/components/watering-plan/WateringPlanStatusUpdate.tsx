import { useCallback, useState } from 'react'
import BackLink from '../general/links/BackLink'
import {
  WateringPlan,
  WateringPlanStatus,
  WateringPlanUpdate,
} from '@green-ecolution/backend-client'
import { wateringPlanIdQuery, wateringPlanQuery } from '@/api/queries'
import { format } from 'date-fns'
import PrimaryButton from '../general/buttons/PrimaryButton'
import FormError from '../general/form/FormError'
import Select from '../general/form/types/Select'
import {
  getWateringPlanStatusDetails,
  WateringPlanStatusOptions,
} from '@/hooks/details/useDetailsForWateringPlanStatus'
import Pill from '../general/Pill'
import Textarea from '../general/form/types/Textarea'
import {
  WateringPlanFinishedForm,
  wateringPlanFinishedSchema,
  WateringPlanCancelForm,
  wateringPlanCancelSchema,
} from '@/schema/wateringPlanSchema'
import { zodResolver } from '@hookform/resolvers/zod'
import { SubmitHandler, useFieldArray, useForm } from 'react-hook-form'
import Input from '../general/form/types/Input'
import SelectedCard from '../general/cards/SelectedCard'
import { useMutation, useQueryClient, useSuspenseQuery } from '@tanstack/react-query'
import { wateringPlanApi } from '@/api/backendApi'
import { useNavigate } from '@tanstack/react-router'
import useToast from '@/hooks/useToast'

interface WateringPlanStatusUpdateProps {
  wateringPlanId: string
}

const WateringPlanStatusUpdate = ({ wateringPlanId }: WateringPlanStatusUpdateProps) => {
  const { data: loadedData } = useSuspenseQuery(wateringPlanIdQuery(wateringPlanId))
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const showToast = useToast()
  const [status, setStatus] = useState(getWateringPlanStatusDetails(loadedData.status))

  const { mutate, isError, error } = useMutation({
    mutationFn: (wateringPlan: WateringPlanUpdate) =>
      wateringPlanApi.updateWateringPlan({
        id: wateringPlanId,
        body: wateringPlan,
      }),

    onSuccess: (data: WateringPlan) => {
      queryClient
        .invalidateQueries(wateringPlanIdQuery(String(data.id)))
        .catch((error) => console.error('Invalidate "waterinPlanIdQuery" failed', error))
      queryClient
        .invalidateQueries(wateringPlanQuery())
        .catch((error) => console.error('Invalidate "wateringPlanQuery" failed:', error))

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
  const statusDetails = getWateringPlanStatusDetails(loadedData.status)

  const formByStatus = useCallback(
    (status: WateringPlanStatus) => {
      const onSubmitFinished: SubmitHandler<WateringPlanFinishedForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.WateringPlanStatusFinished,
          evaluation: data.evaluation,
          transporterId: loadedData.transporter.id,
          treeClusterIds: loadedData.treeclusters.map((cluster) => cluster.id),
        })
      }

      const onSubmitCancel: SubmitHandler<WateringPlanCancelForm> = (data) => {
        mutate({
          ...loadedData,
          status: WateringPlanStatus.WateringPlanStatusCanceled,
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
            <PrimaryButton
              onClick={() => onSubmitOtherStatus(status)}
              type="submit"
              label="Speichern"
              className="mt-10"
            />
          )
      }
    },
    [loadedData, wateringPlanId, mutate],
  )

  return (
    <>
      <article className="2xl:w-4/5">
        <BackLink
          label="Zurück zm Einsatzplan"
          link={{
            to: `/watering-plans/$wateringPlanId`,
            params: { wateringPlanId },
          }}
        />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Status vom Einsatzplan {date} bearbeiten
        </h1>
        <p className="space-x-3 mb-5">
          <strong>Aktueller Status:</strong>
          <Pill label={statusDetails.label} theme={statusDetails.color} />
        </p>
        <p>
          Der Status eines Einsatzes beschreibt, ob der Einsatz beispielsweise aktiv ausgeführt
          wird, beendet ist oder abgebrochen wurde. Diese Angabe hilft dabei die erstellen Einsätze
          zu kategorisieren und eine Auswertung anzulegen. Sobald ein Einsatz beendet wird, kann
          zudem angegeben werden, mit wie viel Wasser die zugehörigen Bewässerungsgruppen bewässert
          wurden.
        </p>
      </article>

      <section className="mt-10">
        <div className="space-y-6 md:w-1/2">
          <Select
            options={WateringPlanStatusOptions}
            placeholder="Wählen Sie einen Status aus"
            label="Status des Einsatzes"
            required
            value={status.value}
            onChange={(e) => {
              console.log(e.target.value)
              setStatus(getWateringPlanStatusDetails(e.target.value as WateringPlanStatus))
            }}
          />
        </div>
        {formByStatus(status.value)}
        <FormError show={isError} error={error?.message} />
      </section>
    </>
  )
}

interface CancelPlanProps {
  onSubmit: SubmitHandler<WateringPlanCancelForm>
}

const CancelWateringPlan = ({ onSubmit }: CancelPlanProps) => {
  const {
    register,
    handleSubmit,
    formState: { isValid, errors },
  } = useForm({
    resolver: zodResolver(wateringPlanCancelSchema),
  })

  return (
    <form className="md:w-1/2" onSubmit={handleSubmit(onSubmit)}>
      <Textarea
        placeholder="Warum wurde der Einsatz abgebrochen?"
        label="Grund des Abbruchs"
        error={errors.cancellationNote?.message}
        required
        {...register('cancellationNote')}
      />

      <PrimaryButton type="submit" label="Speichern" disabled={!isValid} className="mt-10" />
    </form>
  )
}

interface FinishedPlanProps {
  onSubmit: SubmitHandler<WateringPlanFinishedForm>
  wateringPlanId: string
  loadedData: WateringPlan
}

const FinishedWateringPlan = ({ wateringPlanId, onSubmit, loadedData }: FinishedPlanProps) => {
  const {
    register,
    handleSubmit,
    formState: { isValid },
    control,
  } = useForm({
    resolver: zodResolver(wateringPlanFinishedSchema),
    defaultValues: {
      evaluation: loadedData.treeclusters.map((cluster) => ({
        consumedWater: (cluster.treeIds?.length ?? 1) * 80,
        treeClusterId: cluster.id,
        wateringPlanId: Number(wateringPlanId),
      })),
    },
  })

  const { fields } = useFieldArray({
    control,
    name: 'evaluation',
  })

  return (
    <form className="space-y-6" onSubmit={handleSubmit(onSubmit)}>
      <fieldset className="mt-6">
        <legend className="block font-semibold text-dark-800 mb-2.5">
          Wasservergabe pro Bewässerungsgruppe:
        </legend>
        <p className="-mt-2 text-sm text-dark-600 mb-2.5">
          Die Standardwerte ergeben sich aus 80 Litern pro Baum einer Bewässerungsgruppe.
        </p>
        <ul className="space-y-5">
          {fields.map((field, index) => (
            <li key={field.treeClusterId} className="grid grid-cols-1 gap-y-2 md:grid-cols-2">
              <SelectedCard type="cluster" id={loadedData?.treeclusters[index].id} />
              <div className="relative flex flex-wrap items-center md:mb-3 md:ml-6">
                <Input
                  type="number"
                  label="Liter"
                  defaultValue={field.consumedWater}
                  small
                  hideLabel
                  {...register(`evaluation.${index}.consumedWater`)}
                />
                <span className="absolute left-[8.5rem] top-1/2 -translate-y-1/2 ml-2">Liter</span>
              </div>
            </li>
          ))}
        </ul>
      </fieldset>

      <PrimaryButton type="submit" label="Speichern" disabled={!isValid} className="mt-10" />
    </form>
  )
}

export default WateringPlanStatusUpdate
