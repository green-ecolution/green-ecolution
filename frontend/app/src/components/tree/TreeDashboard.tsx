import BackLink from '../general/links/BackLink'
import { Badge, Tabs, TabsList, TabsTrigger, TabsContent } from '@green-ecolution/ui'
import GeneralLink from '../general/links/GeneralLink'
import ButtonLink from '../general/links/ButtonLink'
import { File, Info, Pencil } from 'lucide-react'
import TreeIcon from '../icons/Tree'
import SensorIcon from '../icons/Sensor'
import TabWateringStatus from './TabWateringStatus'
import TabGeneralData from './TabGeneralData'
import TabSensorData from './TabSensorData'
import { Tree, TreeCluster } from '@green-ecolution/backend-client'

interface TreeDashboardProps {
  tree: Tree
  treeCluster?: TreeCluster
}

const TreeDashboard = ({ tree, treeCluster }: TreeDashboardProps) => {
  return (
    <>
      <BackLink link={{ to: '/trees' }} label="Zu allen Bäumen" />
      <article className="space-y-6 2xl:space-y-0 2xl:flex 2xl:items-center 2xl:space-x-10">
        <div className="2xl:w-4/5">
          <h1 className="font-lato font-bold text-3xl mb-4 flex flex-wrap items-center gap-4 lg:text-4xl xl:text-5xl">
            Baum: {tree.number}
            <Badge variant="outline-green-light" size="lg">
              {tree.provider ?? 'manuell erstellt'}
            </Badge>
          </h1>
          {tree.treeClusterId && treeCluster ? (
            <p className="text-dark-600 text-lg">
              <span>Bewässerungsgruppe: {treeCluster?.name}</span>
              {', '}
              <span>
                Standort: {treeCluster?.address}, {treeCluster?.region?.name}
              </span>
            </p>
          ) : (
            <p className="text-dark-600 text-lg">
              Dieser Baum ist keiner Bewässerungsgruppe zugeordnet.
            </p>
          )}
          {tree.description && <p>{tree.description}</p>}
          <div className="flex mt-4 flex-wrap gap-x-10">
            <GeneralLink
              label="Auf der Karte anzeigen"
              link={{
                to: '/map',
                search: {
                  lat: tree.latitude,
                  lng: tree.longitude,
                  zoom: 18,
                  tree: tree.id,
                },
              }}
            />
            {tree.treeClusterId && treeCluster && (
              <GeneralLink
                label="Zur Bewässerungsgruppe"
                link={{
                  to: `/treecluster/$treeclusterId`,
                  params: { treeclusterId: String(tree.treeClusterId) },
                }}
              />
            )}
          </div>
        </div>
        <ButtonLink
          icon={Pencil}
          iconClassName="stroke-1"
          label="Baum bearbeiten"
          color="grey"
          link={{
            to: `/trees/$treeId/edit`,
            params: { treeId: String(tree.id) },
          }}
        />
      </article>

      {tree?.sensor ? (
        <Tabs defaultValue="watering" className="mt-10">
          <TabsList>
            <TabsTrigger value="watering">
              <TreeIcon className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">
                Bewässerungsdaten
              </span>
            </TabsTrigger>
            <TabsTrigger value="general">
              <File className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">
                Allgemeine Daten
              </span>
            </TabsTrigger>
            <TabsTrigger value="sensor">
              <SensorIcon className="w-5 h-5" />
              <span className="hidden group-data-[state=active]:block lg:block">Sensordaten</span>
            </TabsTrigger>
          </TabsList>
          <TabsContent value="watering">
            <TabWateringStatus tree={tree} />
          </TabsContent>
          <TabsContent value="general">
            <TabGeneralData tree={tree} />
          </TabsContent>
          <TabsContent value="sensor">
            <TabSensorData tree={tree} />
          </TabsContent>
        </Tabs>
      ) : (
        <section className="mt-10">
          <div className="bg-white mb-10 border-dark-50 shadow-cards h-full p-6 rounded-xl group flex flex-col gap-4 border">
            <h3 className="font-lato text-lg text-dark font-semibold flex items-center gap-x-3">
              <Info className="text-dark-600 w-5 h-5" />
              Hinweis: Dieser Baum ist nicht mit einem Sensor ausgestattet.
            </h3>
            <p>
              Dieser Baum wurde bisher nicht mit einem Sensor ausgestattet, sodass keine
              Informationen über den aktuellen Bewässerungszustand angezeigt werden können. Aus
              diesem Grund wird der Bewässerungszustand als unbekannt ausgezeichnet.
            </p>
          </div>
          <TabGeneralData tree={tree} />
        </section>
      )}
    </>
  )
}

export default TreeDashboard
