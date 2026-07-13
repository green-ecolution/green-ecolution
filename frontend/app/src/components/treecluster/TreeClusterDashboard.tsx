import { useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { ChevronDown, Pencil, Trash2 } from 'lucide-react'
import TreeCard from '@/components/general/cards/TreeCard'
import ClusterSignalCard from './ClusterSignalCard'
import EntityDetailHeader from '@/components/general/EntityDetailHeader'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import GeneralLink from '../general/links/GeneralLink'
import createToast from '@/hooks/createToast'
import { format } from 'date-fns'
import {
  Alert,
  AlertIcon,
  AlertContent,
  AlertTitle,
  AlertDescription,
  Button,
  ButtonGroup,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  StatusCard,
} from '@green-ecolution/ui'
import { clusterApi, type TreeCluster, type Tree } from '@/api/backendApi'

interface TreeClusterDashboardProps {
  treecluster: TreeCluster
}

const TreeClusterDashboard = ({ treecluster }: TreeClusterDashboardProps) => {
  const navigate = useNavigate()
  const showToast = createToast()
  const [confirmDelete, setConfirmDelete] = useState(false)
  const wateringStatus = getWateringStatusDetails(treecluster.wateringStatus)
  const lastWateredDate = treecluster.lastWatered
    ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy')
    : 'Keine Angabe'

  const handleDelete = () => {
    clusterApi
      .deleteCluster({ clusterId: treecluster.id.toString() })
      .then(() => navigate({ to: '/treecluster', search: { page: 1 } }))
      .then(() => showToast('Die Bewässerungsgruppe wurde gelöscht.'))
      .catch((error) => {
        console.error('Delete failed:', error)
        showToast('Die Bewässerungsgruppe konnte nicht gelöscht werden.', 'error')
      })
  }

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/treecluster' }, label: 'Zu allen Bewässerungsgruppen' }}
        title={<>Bewässerungsgruppe: {treecluster.name}</>}
        actions={
          <ButtonGroup>
            <Button variant="outline" asChild>
              <Link
                to="/map/treecluster/edit/$treeclusterId"
                params={{ treeclusterId: treecluster.id.toString() }}
              >
                Gruppe bearbeiten
                <Pencil className="stroke-1" />
              </Link>
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="outline"
                  size="icon"
                  aria-label="Weitere Aktionen"
                  className="[&_svg]:size-4 [&_svg]:transition-transform [&_svg]:duration-300 data-[state=open]:[&_svg]:rotate-180"
                >
                  <ChevronDown />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="min-w-[12rem]">
                <DropdownMenuItem
                  className="gap-2 px-3 py-2 text-destructive focus:bg-destructive/10 focus:text-destructive"
                  onSelect={() => setConfirmDelete(true)}
                >
                  <Trash2 />
                  Gruppe löschen
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </ButtonGroup>
        }
      >
        {treecluster.description && <p className="mb-4">{treecluster.description}</p>}
        {treecluster.trees?.length === 0 ? (
          <Alert variant="destructive" className="flex gap-4">
            <AlertIcon variant="destructive" />
            <AlertContent>
              <AlertTitle>Keine Bäume zugewiesen</AlertTitle>
              <AlertDescription>
                Diese Baumgruppe enthält keine Bäume und hat daher keinen Standort.
              </AlertDescription>
            </AlertContent>
          </Alert>
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
      </EntityDetailHeader>

      <section className="mt-10">
        <ul className="flex flex-col gap-y-5 md:grid md:gap-5 md:grid-cols-2 xl:grid-cols-4">
          <li className="h-full">
            <StatusCard
              status={wateringStatus.color}
              indicator="dot"
              label="Bewässerungszustand (ø)"
              value={wateringStatus.label}
              description={wateringStatus.description}
            />
          </li>
          <li className="h-full">
            <StatusCard
              label="Baumanzahl in der Gruppe"
              value={
                treecluster.trees?.length
                  ? `${treecluster.trees.length} ${treecluster.trees.length > 1 ? 'Bäume' : 'Baum'}`
                  : 'Keine Bäume'
              }
              description="Nicht alle Bäume haben Sensoren, da Rückschlüsse möglich sind."
            />
          </li>
          <li className="h-full">
            <StatusCard
              label="Standort der Gruppe"
              value={`${treecluster.address}, ${treecluster.region?.name ?? '-'}`}
            />
          </li>
          <li className="h-full">
            <StatusCard
              label="Datum der letzten Bewässerung"
              value={lastWateredDate}
              description="Wird aktualisiert, sobald ein Einsatzplan mit dieser Gruppe als »Beendet« markiert wird."
            />
          </li>
        </ul>
      </section>

      <ClusterSignalCard treecluster={treecluster} />

      <section className="mt-16">
        <h2 className="text-xl font-bold font-lato mb-10">Alle zugehörigen Bäume</h2>

        <header className="hidden border-b pb-2 text-sm text-dark-800 px-6 border-b-dark-200 mb-5 lg:grid lg:grid-cols-[1.5fr_2fr_1fr] lg:gap-5">
          <p>Status</p>
          <p>Baumart</p>
          <p>Baumnummer</p>
        </header>

        <ul className="flex flex-col gap-y-5">
          {treecluster.trees?.length === 0 ? (
            <li className="text-center text-dark-600 mt-4">
              <p>Der Bewässerungsgruppe wurden keine Bäume hinzugefügt.</p>
            </li>
          ) : (
            treecluster.trees?.map((tree: Tree) => (
              <li key={tree.id}>
                <TreeCard tree={tree} showTreeClusterInfo={false} />
              </li>
            ))
          )}
        </ul>
      </section>

      <DeleteConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title="Bewässerungsgruppe löschen?"
        description="Möchtest du die Bewässerungsgruppe wirklich löschen? Die zugehörigen Bäume bleiben erhalten."
        onConfirm={handleDelete}
      />
    </>
  )
}

export default TreeClusterDashboard
