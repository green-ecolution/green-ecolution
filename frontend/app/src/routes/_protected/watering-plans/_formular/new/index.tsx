import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { WateringPlanStatus } from '@/api/backendApi'
import { DefaultValues, FormProvider, SubmitHandler } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import BackLink from '@/components/general/links/BackLink'
import { userRoleQuery, vehicleQuery } from '@/api/queries'
import { WateringPlanForm, wateringPlanSchemaBase } from '@/schema/wateringPlanSchema'
import FormForWateringPlan from '@/components/general/form/FormForWateringPlan'
import useStore from '@/store/store'
import { useWateringPlanForm } from '@/hooks/form/useWateringPlanForm'
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

export const Route = createFileRoute('/_protected/watering-plans/_formular/new/')({
  loader: () => {
    const { data } = safeJsonStorageParse('create-wateringplan', { schema: wateringPlanSchemaBase })
    return {
      formState: data,
    }
  },
  component: NewWateringPlan,
})

const defaultForm: DefaultValues<WateringPlanForm> = {
  date: new Date(),
  description: '',
  transporterId: -1,
  trailerId: undefined,
  clusterIds: [],
  status: WateringPlanStatus.WateringPlanStatusPlanned,
  driverIds: [],
}

function NewWateringPlan() {
  const { formState } = Route.useLoaderData()
  const navigate = useNavigate({ from: Route.fullPath })
  const { data: users } = useSuspenseQuery(userRoleQuery('tbz'))
  const { data: trailers } = useSuspenseQuery(
    vehicleQuery({
      type: 'trailer',
    }),
  )
  const { data: transporters } = useSuspenseQuery(
    vehicleQuery({
      type: 'transporter',
    }),
  )
  const { mutate, isError, error, form, navigationBlocker } = useWateringPlanForm('create', {
    initForm: formState ?? defaultForm,
  })
  const { getValues } = form

  const mapCenter = useStore((state) => state.mapCenter)
  const mapZoom = useStore((state) => state.mapZoom)
  const mapPosition = { lat: mapCenter[0], lng: mapCenter[1], zoom: mapZoom }

  const onSubmit: SubmitHandler<WateringPlanForm> = (data) => {
    mutate({
      ...data,
      date: data.date.toISOString(),
      trailerId: data.trailerId && data.trailerId !== -1 ? data.trailerId : undefined,
      userIds: data.driverIds,
      treeClusterIds: data.clusterIds,
    })
  }

  const navigateToClusterSelect = () => {
    navigate({
      to: '/map/watering-plan/select/cluster',
      search: {
        lat: mapPosition.lat,
        lng: mapPosition.lng,
        zoom: mapPosition.zoom,
        transporterId: getValues('transporterId'),
        trailerId: getValues('trailerId'),
        formType: 'create',
        clusterIds: form.getValues('clusterIds'),
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <BackLink link={{ to: '/watering-plans' }} label="Zu allen Einsatzplänen" />
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Neuen Einsatzplan erstellen
        </h1>
        <p className="mb-5">
          Ein Einsatzplan bildet eine Bewässerungsroute ab, indem ihr ein Fahrzeug, einen Anhänger,
          Mitarbeitende und anzufahrende Bewässerungsgruppen zugewiesen werden können. Ein neu
          erstellter Einsatzplan wird automatisch als »geplant« eingestuft. Anhand der
          Bewässerungsgruppen und die Anzahl der Bäume wird berechnet, wie viel Wasser zum
          bewässsern benötigt wird. Ein Einsatzplan startet immer an der Hauptzentrale des TBZ in
          der Schleswiger Straße in Flensburg.
        </p>
      </article>

      <section className="mt-10">
        <FormProvider {...form}>
          <FormForWateringPlan
            displayError={isError}
            onSubmit={onSubmit}
            trailers={trailers.data}
            transporters={transporters.data}
            users={users.data}
            onAddCluster={navigateToClusterSelect}
            errorMessage={error?.message}
          />
        </FormProvider>
      </section>

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
    </div>
  )
}

export default NewWateringPlan
