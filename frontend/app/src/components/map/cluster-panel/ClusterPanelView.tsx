import { ComponentProps } from 'react'
import { MoveRight, Pencil, X } from 'lucide-react'
import { Badge, Button } from '@green-ecolution/ui'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import type { TreeClusterResponse } from '@/api/backendApi'
import { summarizeTopSpecies } from './clusterPanelUtils'

interface ClusterPanelViewProps {
  treecluster: TreeClusterResponse
  onEdit: () => void
  onClose: () => void
  onOpenDashboard: () => void
}

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
  const treeCount = treecluster.trees.length

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
    </div>
  )
}

export default ClusterPanelView
