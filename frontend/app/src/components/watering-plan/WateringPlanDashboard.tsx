import EntityDetailHeader from '../general/EntityDetailHeader'
import EntityList from '../general/EntityList'
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
import { File, FolderClosed, MoveRight, Route } from 'lucide-react'
import TabGeneralData from './TabGeneralData'
import TreeclusterCard from '../general/cards/TreeclusterCard'
import ButtonLink from '../general/links/ButtonLink'
import { WateringPlan } from '@/api/backendApi'
import { useDownloadGpx } from '@/hooks/useDownloadGpx'
import WateringPlanPreviewRoute from './WateringPlanRoutePreview'

interface WateringPlanDashboardProps {
  wateringPlan: WateringPlan
}

const WateringPlanDashboard = ({ wateringPlan }: WateringPlanDashboardProps) => {
  const statusDetails = getWateringPlanStatusDetails(wateringPlan.status)

  const date = wateringPlan?.date
    ? format(new Date(wateringPlan?.date), 'dd.MM.yyyy')
    : 'Keine Angabe'

  const { mutate: downloadGpx } = useDownloadGpx(wateringPlan.gpxUrl)

  return (
    <>
      <EntityDetailHeader
        breakpoint="xl"
        backLink={{ link: { to: '/watering-plans' }, label: 'Alle Einsatzpläne' }}
        title={<>Einsatzplan für den {date}</>}
        badge={
          <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
            {statusDetails?.label ?? 'Keine Angabe'}
          </Badge>
        }
        editLink={{
          label: 'Einsatz bearbeiten',
          link: {
            to: `/watering-plans/$wateringPlanId/edit`,
            params: { wateringPlanId: String(wateringPlan.id) },
          },
        }}
      >
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
          <Button variant="nav" onClick={() => downloadGpx()} className="p-0 h-auto [&_svg]:size-4">
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
                Sie, ob das ausgewählte Fahrzeug über ausreichend Wasserkapazität für die gewählten
                Bewässerungsgruppen verfügt.
              </AlertDescription>
            </AlertContent>
          </Alert>
        )}
      </EntityDetailHeader>

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
          <EntityList
            items={wateringPlan.treeclusters}
            getKey={(cluster) => cluster.id}
            emptyMessage="Es wurden leider keine Bewässerungsgruppen gefunden."
            renderItem={(cluster) => <TreeclusterCard treecluster={cluster} />}
          />
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
