import { Checkbox, KanbanCard } from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { TreeClusterInListResponse } from '@green-ecolution/backend-client'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

interface ClusterSuggestionCardProps {
  cluster: TreeClusterInListResponse
  selected?: boolean
  /** Omit to render the card read-only — no checkbox, no click-to-select. */
  onSelectedChange?: (selected: boolean) => void
}

const ClusterSuggestionCard = ({
  cluster,
  selected = false,
  onSelectedChange,
}: ClusterSuggestionCardProps) => {
  const getWateringStatusDetails = useWateringStatusDetails()
  const statusDetails = getWateringStatusDetails(cluster.wateringStatus)
  const { t } = useTranslation('wateringPlan')

  const toggleOnCardClick = (event: React.MouseEvent) => {
    // Link keeps navigating, checkbox already toggles itself — don't double-toggle.
    if ((event.target as HTMLElement).closest('a, [role="checkbox"]')) return
    onSelectedChange?.(!selected)
  }

  return (
    <KanbanCard
      className={`flex items-start gap-3 ${onSelectedChange ? 'cursor-pointer' : ''}`}
      onClick={onSelectedChange ? toggleOnCardClick : undefined}
    >
      {onSelectedChange && (
        <Checkbox
          checked={selected}
          onCheckedChange={(checked) => onSelectedChange(checked === true)}
          aria-label={t('board.suggestionCard.selectAriaLabel', { name: cluster.name })}
          className="mt-0.5"
        />
      )}
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
          <span className="tabular-nums">
            {t('board.suggestionCard.treeCount', { count: cluster.treeIds.length })}
          </span>
          <span>·</span>
          <span>{statusDetails.label}</span>
        </p>
      </div>
    </KanbanCard>
  )
}

export default ClusterSuggestionCard
