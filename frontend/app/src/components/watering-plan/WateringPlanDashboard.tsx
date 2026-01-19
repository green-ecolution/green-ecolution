import BackLink from '../general/links/BackLink'
import {
  Badge,
  Button,
  Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  Alert,
  AlertIcon,
  AlertContent,
  AlertDescription,
} from '@green-ecolution/ui'
import {
  getWateringPlanStatusDetails,
  showWateringPlanStatusButton,
} from '@/hooks/details/useDetailsForWateringPlanStatus'
import { format } from 'date-fns'
import { File, FolderClosed, MoveRight, Pencil, Route } from 'lucide-react'
import TabGeneralData from './TabGeneralData'
import TreeClusterList from '../treecluster/TreeClusterList'
import ButtonLink from '../general/links/ButtonLink'
import { useMutation } from '@tanstack/react-query'
import useStore from '@/store/store'
import { basePath, WateringPlan } from '@/api/backendApi'
import createToast from '@/hooks/createToast'
import WateringPlanPreviewRoute from './WateringPlanRoutePreview'
import { isHTTPError } from '@/lib/utils'

interface WateringPlanDashboardProps {
  wateringPlan: WateringPlan
}

const WateringPlanDashboard = ({ wateringPlan }: WateringPlanDashboardProps) => {
  const statusDetails = getWateringPlanStatusDetails(wateringPlan.status)
  const accessToken = useStore((state) => state.token?.accessToken)
  const showToast = createToast()

  const date = wateringPlan?.date
    ? format(new Date(wateringPlan?.date), 'dd.MM.yyyy')
    : 'Keine Angabe'

  const { mutate } = useMutation({
    mutationFn: async () => {
      const resp = await fetch(`${basePath}${wateringPlan.gpxUrl}`, {
        method: 'GET',
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      })

      if (resp.status !== 200) {
        const json: unknown = await resp.json()
        const errorMsg = isHTTPError(json) ? json.error : 'Unbekannter Fehler'
        throw new Error(errorMsg)
      }

      const blob = await resp.blob()

      const objUrl = window.URL.createObjectURL(blob)

      const a = document.createElement('a')
      a.href = objUrl
      a.download = resp.headers.get('Content-Disposition')?.split('filename=')[1] ?? 'route.gpx'
      a.click()

      window.URL.revokeObjectURL(objUrl)
    },
    onError: (error) => {
      showToast(error.message, 'error')
    },
  })

  return (
    <>
      <BackLink link={{ to: '/watering-plans' }} label="Alle Einsatzpläne" />
      <article className="flex flex-col gap-y-6 xl:flex-row xl:items-start xl:gap-x-10">
        <div className="xl:w-4/5">
          <h1 className="font-lato font-bold text-3xl mb-4 flex flex-wrap items-center gap-4 lg:text-4xl xl:text-5xl">
            Einsatzplan für den {date}
            <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
              {statusDetails?.label ?? 'Keine Angabe'}
            </Badge>
          </h1>
          {wateringPlan.description && <p className="mb-4">{wateringPlan.description}</p>}
          <div className="flex flex-wrap gap-4 items-center">
            {showWateringPlanStatusButton(wateringPlan) && (
              <ButtonLink
                link={{
                  to: '/watering-plans/$wateringPlanId/status/edit',
                  params: { wateringPlanId: wateringPlan.id.toString() },
                }}
                label="Status aktualisieren"
                icon={MoveRight}
              />
            )}
            <Button variant="nav" onClick={() => mutate()} className="p-0 h-auto [&_svg]:size-4">
              Route herunterladen
              <MoveRight className="icon-arrow-animate" />
            </Button>
          </div>
          {wateringPlan.distance == 0 && (
            <Alert variant="destructive" className="mt-6 flex items-center gap-3">
              <AlertIcon variant="destructive" />
              <AlertContent>
                <AlertDescription>
                  Die Route für diesen Einsatzplan konnte nicht berechnet werden. Bitte überprüfen
                  Sie, ob das ausgewählte Fahrzeug über ausreichend Wasserkapazität für die
                  gewählten Bewässerungsgruppen verfügt.
                </AlertDescription>
              </AlertContent>
            </Alert>
          )}
        </div>
        <ButtonLink
          icon={Pencil}
          iconClassName="stroke-1"
          label="Einsatz bearbeiten"
          color="grey"
          link={{
            to: `/watering-plans/$wateringPlanId/edit`,
            params: { wateringPlanId: String(wateringPlan.id) },
          }}
        />
      </article>

      <Tabs defaultValue="general" className="mt-10">
        <TabsList>
          <TabsTrigger value="general">
            <File className="w-5 h-5" />
            <span className="hidden group-data-[state=active]:block lg:block">
              Allgemeine Daten
            </span>
          </TabsTrigger>
          <TabsTrigger value="clusters">
            <FolderClosed className="w-5 h-5" />
            <span className="hidden group-data-[state=active]:block lg:block">
              Bewässerungsgruppen
            </span>
          </TabsTrigger>
          {wateringPlan.distance > 0 && (
            <TabsTrigger value="route">
              <Route className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">Route</span>
            </TabsTrigger>
          )}
        </TabsList>
        <TabsContent value="general">
          <TabGeneralData wateringPlan={wateringPlan} />
        </TabsContent>
        <TabsContent value="clusters">
          <TreeClusterList data={wateringPlan.treeclusters} />
        </TabsContent>
        {wateringPlan.distance > 0 && (
          <TabsContent value="route">
            <WateringPlanPreviewRoute wateringPlan={wateringPlan} />
          </TabsContent>
        )}
      </Tabs>
    </>
  )
}

export default WateringPlanDashboard
