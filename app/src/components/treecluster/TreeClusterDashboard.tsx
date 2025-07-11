import EntitiesStatusCard from '@/components/general/cards/EntitiesStatusCard'
import GeneralStatusCard from '@/components/general/cards/GeneralStatusCard'
import TreeCard from '@/components/general/cards/TreeCard'
import BackLink from '@/components/general/links/BackLink'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Pencil } from 'lucide-react'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import GeneralLink from '../general/links/GeneralLink'
import { format } from 'date-fns'
import Notice from '../general/Notice'
import { TreeCluster } from '@green-ecolution/backend-client'

interface TreeClusterDashboardProps {
  treecluster: TreeCluster
}

const TreeClusterDashboard = ({ treecluster }: TreeClusterDashboardProps) => {
  const wateringStatus = getWateringStatusDetails(treecluster.wateringStatus)
  const lastWateredDate = treecluster.lastWatered
    ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy')
    : 'Keine Angabe'

  return (
    <>
      <BackLink link={{ to: '/treecluster' }} label="Zu allen Bewässerungsgruppen" />
      <article className="space-y-6 2xl:space-y-0 2xl:flex 2xl:items-center 2xl:space-x-10">
        <div className="2xl:w-4/5">
          <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
            Bewässerungsgruppe: {treecluster.name}
          </h1>
          {treecluster.description && <p className="mb-4">{treecluster.description}</p>}
          {treecluster.trees?.length === 0 ? (
            <Notice
              description="Diese Baumgruppe enthält keine Bäume und hat daher keinen
                Standort."
            />
          ) : (
            <GeneralLink
              link={{
                to: '/map',
                search: {
                  lat: treecluster.latitude,
                  lng: treecluster.longitude,
                  zoom: 16,
                  cluster: treecluster.id,
                },
              }}
              label="Auf der Karte anzeigen"
            />
          )}
        </div>
        <ButtonLink
          icon={Pencil}
          iconClassName="stroke-1"
          label="Gruppe bearbeiten"
          color="grey"
          link={{
            to: '/treecluster/$treeclusterId/edit',
            params: { treeclusterId: treecluster.id.toString() },
          }}
        />
      </article>

      <section className="mt-10">
        <ul className="space-y-5 md:space-y-0 md:grid md:gap-5 md:grid-cols-2 lg:grid-cols-4">
          <li>
            <EntitiesStatusCard statusDetails={wateringStatus} label="Bewässerungszustand (ø)" />
          </li>
          <li>
            <GeneralStatusCard
              overline="Baumanzahl in der Gruppe"
              value={
                treecluster.trees?.length
                  ? `${treecluster.trees.length} ${treecluster.trees.length > 1 ? 'Bäume' : 'Baum'}`
                  : 'Keine Bäume'
              }
              description="Nicht alle Bäume haben Sensoren, da Rückschlüsse möglich sind."
            />
          </li>
          <li>
            <GeneralStatusCard
              overline="Standort der Gruppe"
              value={`${treecluster.address}, ${treecluster.region?.name ?? '-'}`}
            />
          </li>
          <li>
            <GeneralStatusCard
              overline="Datum der letzten Bewässerung"
              value={lastWateredDate}
              description="Wird aktualisiert, sobald ein Einsatzplan mit dieser Gruppe als »Beendet« markiert wird."
            />
          </li>
        </ul>
      </section>

      <section className="mt-16">
        <h2 className="text-xl font-bold font-lato mb-10">Alle zugehörigen Bäume</h2>

        <header className="hidden border-b pb-2 text-sm text-dark-800 px-6 border-b-dark-200 mb-5 lg:grid lg:grid-cols-[1.5fr,2fr,1fr] lg:gap-5">
          <p>Status</p>
          <p>Baumart</p>
          <p>Baumnummer</p>
        </header>

        <ul className="space-y-5">
          {treecluster.trees?.length === 0 ? (
            <li className="text-center text-dark-600 mt-4">
              <p>Der Bewässerungsgruppe wurden keine Bäume hinzugefügt.</p>
            </li>
          ) : (
            treecluster.trees?.map((tree) => (
              <li key={tree.id}>
                <TreeCard tree={tree} showTreeClusterInfo={false} />
              </li>
            ))
          )}
        </ul>
      </section>
    </>
  )
}

export default TreeClusterDashboard
