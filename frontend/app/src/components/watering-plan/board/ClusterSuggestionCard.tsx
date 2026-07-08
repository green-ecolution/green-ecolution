import { Checkbox, KanbanCard } from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import type { TreeClusterInListResponse } from '@green-ecolution/backend-client'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

interface ClusterSuggestionCardProps {
  cluster: TreeClusterInListResponse
  selected: boolean
  onSelectedChange: (selected: boolean) => void
}

const ClusterSuggestionCard = ({
  cluster,
  selected,
  onSelectedChange,
}: ClusterSuggestionCardProps) => {
  const statusDetails = getWateringStatusDetails(cluster.wateringStatus)

  return (
    <KanbanCard className="flex items-start gap-3">
      <Checkbox
        checked={selected}
        onCheckedChange={(checked) => onSelectedChange(checked === true)}
        aria-label={`${cluster.name} für Einsatzplan auswählen`}
        className="mt-0.5"
      />
      <div className="min-w-0">
        <Link
          to="/treecluster/$treeclusterId"
          params={{ treeclusterId: cluster.id.toString() }}
          className="font-lato font-semibold text-dark hover:underline"
        >
          {cluster.name}
        </Link>
        <p className="mt-0.5 flex items-center gap-1.5 text-sm text-dark-600">
          <span
            aria-hidden
            className="size-2 shrink-0 rounded-full"
            style={{ backgroundColor: statusDetails.colorHex }}
          />
          <span className="tabular-nums">{cluster.treeIds.length} Bäume</span>
          <span>·</span>
          <span>{statusDetails.label}</span>
        </p>
      </div>
    </KanbanCard>
  )
}

export default ClusterSuggestionCard
