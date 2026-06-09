import { ComponentProps } from 'react'
import { MoveRight, Pencil, RadioTower, X } from 'lucide-react'
import { format, formatDistanceToNow } from 'date-fns'
import { de } from 'date-fns/locale'
import { Badge, Button, StatusCard } from '@green-ecolution/ui'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { roundTo } from '@/lib/utils'
import Tree from '@/components/icons/Tree'
import type { TreeClusterResponse } from '@/api/backendApi'
import { latestSensorReading, sortTreesSensorFirst, summarizeTopSpecies } from './clusterPanelUtils'

interface ClusterPanelViewProps {
  treecluster: TreeClusterResponse
  onEdit: () => void
  onClose: () => void
  onOpenDashboard: () => void
}

interface SensorPayload {
  temperature?: number
}

const PREVIEW_COUNT = 3

const FILLED_BADGE: Record<string, ComponentProps<typeof Badge>['variant']> = {
  'outline-red': 'error',
  'outline-yellow': 'warning',
  'outline-green-light': 'success',
  'outline-green-dark': 'success',
}

const ClusterPanelView = ({
  treecluster,
  onEdit,
  onClose,
  onOpenDashboard,
}: ClusterPanelViewProps) => {
  const status = getWateringStatusDetails(treecluster.wateringStatus)
  const species = summarizeTopSpecies(treecluster.trees)
  const sortedTrees = sortTreesSensorFirst(treecluster.trees)
  const previewTrees = sortedTrees.slice(0, PREVIEW_COUNT)
  const remaining = sortedTrees.length - previewTrees.length
  const treeCount = treecluster.trees.length
  const moisturePercent = Math.round(treecluster.moistureLevel * 100)
  const reading = latestSensorReading(treecluster.trees)
  const temperatureValue = (reading?.data as SensorPayload | undefined)?.temperature
  const temperature =
    typeof temperatureValue === 'number' ? `${roundTo(temperatureValue, 1)} °C` : 'Keine Daten'
  const lastMeasurement = reading
    ? formatDistanceToNow(new Date(reading.createdAt), { addSuffix: true, locale: de })
    : 'Keine Daten'
  const lastWatered = treecluster.lastWatered
    ? format(new Date(treecluster.lastWatered), 'dd.MM.yyyy')
    : 'Keine Angabe'

  return (
    <div className="flex flex-col gap-y-5">
      <header className="flex items-center justify-between gap-3">
        <Badge variant={FILLED_BADGE[status.color] ?? 'muted'} size="lg" className="gap-1.5">
          <span
            className="size-1.5 rounded-full"
            style={{ backgroundColor: status.colorHex }}
            aria-hidden="true"
          />
          {status.label}
        </Badge>
        <div className="flex items-center gap-1.5">
          <Button
            variant="ghost"
            size="icon"
            aria-label="Gruppe bearbeiten"
            className="rounded-full bg-dark-50 text-dark-500 hover:bg-dark-100 hover:text-green-dark"
            onClick={onEdit}
          >
            <Pencil className="stroke-[1.5]" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Seitenansicht schließen"
            className="hidden rounded-full bg-dark-50 text-dark-500 hover:bg-dark-100 hover:text-dark-700 lg:flex"
            onClick={onClose}
          >
            <X />
          </Button>
        </div>
      </header>

      <div className="space-y-1.5">
        <h2 className="font-lato text-3xl font-bold leading-tight text-dark-900">
          {treecluster.name}
        </h2>
        <p className="text-sm text-dark-600">
          {treecluster.address} · {treeCount} {treeCount === 1 ? 'Baum' : 'Bäume'}
          {species && ` · ${species}`}
        </p>
      </div>

      <Button onClick={onOpenDashboard} className="group w-full lg:order-last">
        Zum Dashboard
        <MoveRight className="icon-arrow-animate" />
      </Button>

      <div className="grid grid-cols-2 gap-4">
        <StatusCard
          status={status.color}
          label="Bodenfeuchte"
          value={`${moisturePercent} %`}
          progress={moisturePercent}
          isLarge
        />
        <StatusCard label="Bodentemperatur" value={temperature} isLarge />
        <StatusCard label="Letzte Messung" value={lastMeasurement} />
        <StatusCard label="Letzte Bewässerung" value={lastWatered} />
      </div>

      <section>
        <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-dark-500">
          Bäume in dieser Gruppe · {treeCount}
        </p>
        <ul className="flex flex-col">
          {previewTrees.map((tree) => (
            <li
              key={tree.id}
              data-testid="cluster-panel-tree"
              className="flex items-center gap-3 border-b border-dark-100 py-3 text-sm last:border-0"
            >
              {tree.sensor ? (
                <RadioTower className="size-4 shrink-0 text-green-dark" />
              ) : (
                <Tree className="size-4 shrink-0 text-dark-400" />
              )}
              <span className="min-w-0 flex-1 truncate">
                <span className="font-medium text-dark-800">{tree.species}</span>
                <span className="text-dark-500"> · ID {tree.number}</span>
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
            className="mt-3 cursor-pointer text-sm font-semibold text-green-dark transition-colors hover:text-green-dark-700"
          >
            + {remaining} weitere {remaining === 1 ? 'Baum' : 'Bäume'} · alle anzeigen
          </button>
        )}
      </section>
    </div>
  )
}

export default ClusterPanelView
