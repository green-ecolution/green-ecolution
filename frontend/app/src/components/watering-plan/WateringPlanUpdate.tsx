import { useSuspenseQuery } from '@tanstack/react-query'
import BackLink from '../general/links/BackLink'
import { useInitFormQuery } from '@/hooks/form/useInitForm'
import { userRoleQuery, vehicleQuery, wateringPlanIdQuery } from '@/api/queries'
import { format } from 'date-fns'
import FormForWateringPlan from '../general/form/FormForWateringPlan'
import { useNavigate } from '@tanstack/react-router'
import { Route } from '@/routes'
import useStore from '@/store/store'
import GeneralLink from '../general/links/GeneralLink'
import { showWateringPlanStatusButton } from '@/hooks/details/useDetailsForWateringPlanStatus'
import LoadingInfo from '../general/error/LoadingInfo'
import { Suspense } from 'react'
import DeleteSection from '../treecluster/DeleteSection'
import { wateringPlanApi } from '@/api/backendApi'
import { useWateringPlanForm } from '@/hooks/form/useWateringPlanForm'
import { WateringPlanForm, wateringPlanSchemaBase } from '@/schema/wateringPlanSchema'
import { FormProvider, SubmitHandler } from 'react-hook-form'
import { safeJsonStorageParse } from '@/lib/utils'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'

interface WateringPlanUpdateProps {
  wateringPlanId: string
}

const WateringPlanUpdate = ({ wateringPlanId }: WateringPlanUpdateProps) => {
  const { data: formState } = safeJsonStorageParse('update-wateringplan', {
    schema: wateringPlanSchemaBase,
  })
  const { initForm, loadedData } = useInitFormQuery(
    wateringPlanIdQuery(wateringPlanId),
    (data) =>
      formState ?? {
        date: new Date(data.date),
        description: data.description,
        transporterId: data.transporter.id,
        trailerId: data.trailer?.id,
        clusterIds: data.treeclusters.map((cluster) => cluster.id),
        status: data.status,
        driverIds: data.userIds,
      },
  )
  const { mutate, isError, error, form, navigationBlocker } = useWateringPlanForm('update', {
    wateringPlanId,
    initForm,
  })

  const navigate = useNavigate({ from: Route.fullPath })
  const date = loadedData?.date ? format(new Date(loadedData?.date), 'dd.MM.yyyy') : 'Keine Angabe'

  const { data: users } = useSuspenseQuery(userRoleQuery('tbz'))
  const { data: trailers } = useSuspenseQuery(vehicleQuery({ type: 'trailer' }))
  const { data: transporters } = useSuspenseQuery(vehicleQuery({ type: 'transporter' }))

  const onSubmit: SubmitHandler<WateringPlanForm> = (data) => {
    mutate({
      ...data,
      date: data.date.toISOString(),
      trailerId: data.trailerId && data.trailerId !== -1 ? data.trailerId : undefined,
      treeClusterIds: data.clusterIds,
      userIds: data.driverIds,
      cancellationNote: '', // TODO: why cancel note in update ???
    })
  }

  const mapCenter = useStore((state) => state.mapCenter)
  const mapZoom = useStore((state) => state.mapZoom)
  const mapPosition = { lat: mapCenter[0], lng: mapCenter[1], zoom: mapZoom }

  const navigateToClusterSelect = () => {
    navigate({
      to: '/map/watering-plan/select/cluster',
      search: {
        lat: mapPosition.lat,
        lng: mapPosition.lng,
        zoom: mapPosition.zoom,
        wateringPlanId: Number(wateringPlanId),
        trailerId: form.getValues('trailerId'),
        transporterId: form.getValues('transporterId'),
        clusterIds: form.getValues('clusterIds'),
        formType: 'update',
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const handleDeleteWateringPlan = () => {
    return wateringPlanApi.deleteWateringPlan({
      id: Number(wateringPlanId),
    })
  }

  return (
    <>
      <article className="2xl:w-4/5">
        <BackLink
          label="Zurück zum Einsatzplan"
          link={{
            to: `/watering-plans/$wateringPlanId`,
            params: { wateringPlanId: wateringPlanId?.toString() },
          }}
        />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Einsatzplan für den {date} bearbeiten
        </h1>
        <p>
          Der Einsatzplan kann in dieser Ansicht editiert werden, falls bestimmte Daten nicht mehr
          stimmen oder z. B. bestimmte Informationen zusätzlich hinterlegt werden müssen. Ein
          Einsatzplan erfordert immer mindestens eine Bewässerungsgruppe, die angefahren werden soll
          und mindestens eine:n Mitarbeiter:in, die den Einatz durchführen soll. Zudem muss ein
          Fahrzeug hinterlegt werden.
        </p>
        {showWateringPlanStatusButton(loadedData) && (
          <p className="mt-5 flex flex-wrap gap-x-4">
            Der Status eines Einsatzes kann seperat editiert werden.
            <GeneralLink
              link={{
                to: `/watering-plans/$wateringPlanId/status/edit`,
                params: { wateringPlanId: String(loadedData.id) },
              }}
              label="Status aktualisieren"
            />
          </p>
        )}
      </article>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForWateringPlan
            displayError={isError}
            onSubmit={onSubmit}
            users={users.data}
            trailers={trailers.data}
            transporters={transporters.data}
            onAddCluster={navigateToClusterSelect}
            errorMessage={error?.message}
          />
        </FormProvider>
      </section>

      <Suspense fallback={<LoadingInfo label="Der Einsatzplan wird gelöscht" />}>
        <DeleteSection
          mutationFn={handleDeleteWateringPlan}
          entityName="der Einsatzplan"
          redirectUrl={{ to: '/watering-plans' }}
        />
      </Suspense>

      <AlertDialog
        open={navigationBlocker.isModalOpen}
        onOpenChange={(open) => !open && navigationBlocker.closeModal()}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Seite verlassen?</AlertDialogTitle>
            <AlertDialogDescription>{navigationBlocker.message}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={navigationBlocker.closeModal}>
              Abbrechen
              <X />
            </AlertDialogCancel>
            <AlertDialogAction onClick={navigationBlocker.confirmLeave}>
              Verlassen
              <MoveRight />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

export default WateringPlanUpdate
