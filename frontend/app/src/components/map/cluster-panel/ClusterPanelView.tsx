import { Pencil } from 'lucide-react'
import { format } from 'date-fns'
import { Badge, Button, StatusCard } from '@green-ecolution/ui'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import type { TreeClusterResponse } from '@/api/backendApi'
import { sortTreesSensorFirst, summarizeTopSpecies } from './clusterPanelUtils'

interface ClusterPanelViewProps {
  treecluster: TreeClusterResponse
  onEdit: () => void
  onOpenDashboard: () => void
}

const PREVIEW_COUNT = 3

const ClusterPanelView = ({ treecluster, onEdit, onOpenDashboard }: ClusterPanelViewProps) => {
  const status = getWateringStatusDetails(treecluster.wateringStatus)
  const species = summarizeTopSpecies(treecluster.trees)
  const sortedTrees = sortTreesSensorFirst(treecluster.trees)
  const previewTrees = sortedTrees.slice(0, PREVIEW_COUNT)
  const remaining = sortedTrees.length - previewTrees.length
  const lastWatered = treecluster.lastWatered
    ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy')
    : 'Keine Angabe'

  return (
    <div className="flex flex-col gap-y-6">
      <div className="flex items-start justify-between gap-4">
        <Badge variant={status.color}>{status.label}</Badge>
        <Button variant="ghost" size="icon" aria-label="Gruppe bearbeiten" onClick={onEdit}>
          <Pencil className="stroke-1" />
        </Button>
      </div>

      <div>
        <h2 className="font-lato font-bold text-2xl">{treecluster.name}</h2>
        <p className="text-sm text-dark-700">
          {treecluster.address} · {treecluster.trees.length}{' '}
          {treecluster.trees.length === 1 ? 'Baum' : 'Bäume'}
          {species && ` · ${species}`}
        </p>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <StatusCard label="Bodenfeuchte" value={`${treecluster.moistureLevel} %`} />
        <StatusCard
          status={status.color}
          indicator="dot"
          label="Bewässerungszustand"
          value={status.label}
        />
        <StatusCard label="Bäume in der Gruppe" value={treecluster.trees.length} />
        <StatusCard label="Letzte Bewässerung" value={lastWatered} />
      </div>

      <div>
        <p className="text-sm font-semibold text-dark-800 mb-3">
          Bäume in dieser Gruppe · {treecluster.trees.length}
        </p>
        <ul className="flex flex-col gap-y-2">
          {previewTrees.map((tree) => (
            <li
              key={tree.id}
              data-testid="cluster-panel-tree"
              className="flex items-center justify-between gap-3 text-sm"
            >
              <span>
                {tree.species} · ID {tree.number}
              </span>
              {tree.sensor ? (
                <Badge variant="success">Sensor-Baum</Badge>
              ) : (
                <span className="text-dark-600">kein Sensor</span>
              )}
            </li>
          ))}
        </ul>
        {remaining > 0 && (
          <button
            type="button"
            onClick={onOpenDashboard}
            className="mt-3 text-sm font-semibold text-green-dark hover:underline"
          >
            + {remaining} weitere {remaining === 1 ? 'Baum' : 'Bäume'} · alle anzeigen
          </button>
        )}
      </div>

      <Button onClick={onOpenDashboard} className="w-full">
        Zum Dashboard
      </Button>
    </div>
  )
}

export default ClusterPanelView
