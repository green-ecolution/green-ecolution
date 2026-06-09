import { MoveRight, Pencil } from 'lucide-react'
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
  const treeCount = treecluster.trees.length
  const lastWatered = treecluster.lastWatered
    ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy')
    : 'Keine Angabe'

  return (
    <div className="flex flex-col gap-y-6 animate-in fade-in duration-300">
      <header className="flex items-start justify-between gap-3">
        <Badge variant={status.color} className="gap-1.5">
          <span
            className="size-1.5 rounded-full"
            style={{ backgroundColor: status.colorHex }}
            aria-hidden="true"
          />
          {status.label}
        </Badge>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Gruppe bearbeiten"
          className="-mr-2 -mt-1 text-dark-400 hover:text-green-dark"
          onClick={onEdit}
        >
          <Pencil className="stroke-[1.5]" />
        </Button>
      </header>

      <div className="space-y-1">
        <h2 className="font-lato text-2xl font-bold leading-tight text-dark-900">
          {treecluster.name}
        </h2>
        <p className="text-sm text-dark-600">
          {treecluster.address} · {treeCount} {treeCount === 1 ? 'Baum' : 'Bäume'}
          {species && ` · ${species}`}
        </p>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <StatusCard
          status={status.color}
          label="Bodenfeuchte"
          value={`${treecluster.moistureLevel} %`}
          progress={treecluster.moistureLevel}
        />
        <StatusCard
          status={status.color}
          indicator="dot"
          label="Bewässerungszustand"
          value={status.label}
        />
        <StatusCard label="Bäume in der Gruppe" value={treeCount} />
        <StatusCard label="Letzte Bewässerung" value={lastWatered} />
      </div>

      <section>
        <p className="mb-3 text-xs font-semibold uppercase tracking-wide text-dark-500">
          Bäume in dieser Gruppe · {treeCount}
        </p>
        <ul className="flex flex-col">
          {previewTrees.map((tree) => (
            <li
              key={tree.id}
              data-testid="cluster-panel-tree"
              className="flex items-center justify-between gap-3 border-b border-dark-100 py-2.5 text-sm last:border-0"
            >
              <span className="min-w-0 truncate text-dark-800">
                {tree.species} <span className="text-dark-500">· ID {tree.number}</span>
              </span>
              {tree.sensor ? (
                <Badge variant="success" className="shrink-0">
                  Sensor-Baum
                </Badge>
              ) : (
                <span className="shrink-0 text-dark-500">kein Sensor</span>
              )}
            </li>
          ))}
        </ul>
        {remaining > 0 && (
          <button
            type="button"
            onClick={onOpenDashboard}
            className="mt-3 text-sm font-semibold text-green-dark transition-colors hover:text-green-dark-700"
          >
            + {remaining} weitere {remaining === 1 ? 'Baum' : 'Bäume'} · alle anzeigen
          </button>
        )}
      </section>

      <Button onClick={onOpenDashboard} className="group w-full">
        Zum Dashboard
        <MoveRight className="icon-arrow-animate" />
      </Button>
    </div>
  )
}

export default ClusterPanelView
