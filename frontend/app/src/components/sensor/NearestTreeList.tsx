import { clusterQueries } from '@/api/queries'
import type { TreeWithDistance } from '@/api/backendApi'
import { Badge, cn } from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Check, MapPin, TreeDeciduous } from 'lucide-react'
import { useEffect } from 'react'
import { useNumberFormatter } from '@/lib/i18n/useFormatters'

interface NearestTreeListProps {
  trees: TreeWithDistance[]
  selectedTreeId: string | null
  onSelect: (treeId: string) => void
}

const NearestTreeListItem = ({
  entry,
  isSelected,
  onSelect,
}: {
  entry: TreeWithDistance
  isSelected: boolean
  onSelect: () => void
}) => {
  const { t } = useTranslation('sensor')
  const oneDecimal = useNumberFormatter({ maximumFractionDigits: 1 })
  const wholeNumber = useNumberFormatter({ maximumFractionDigits: 0 })
  const { tree, distanceMeters } = entry
  const isAssigned = tree.sensor != null

  const formatDistance = (meters: number): string => {
    if (meters >= 1000) return `${oneDecimal.format(meters / 1000)} km`
    if (meters < 10) return `${oneDecimal.format(meters)} m`
    return `${wholeNumber.format(meters)} m`
  }

  const clusterId = tree.treeClusterId ? String(tree.treeClusterId) : null
  const { data: clusterRes } = useQuery({
    ...clusterQueries.detail(clusterId!),
    enabled: clusterId !== null,
  })

  return (
    <button
      type="button"
      onClick={isAssigned ? undefined : onSelect}
      aria-pressed={!isAssigned && isSelected}
      aria-disabled={isAssigned || undefined}
      disabled={isAssigned}
      className={cn(
        'relative w-full text-left rounded-xl border bg-white p-4 shadow-cards',
        'transition-[color,background-color,border-color,box-shadow,opacity] duration-quick ease-out',
        !isAssigned && 'hover:bg-green-dark-50/50',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-dark focus-visible:ring-offset-2',
        isAssigned && 'opacity-70 cursor-not-allowed',
        !isAssigned && isSelected
          ? 'border-green-dark ring-2 ring-green-dark/20 bg-green-dark-50/30'
          : 'border-dark-100',
      )}
    >
      <div className="flex items-start gap-3">
        <div
          className={cn(
            'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border-2 transition-colors',
            !isAssigned && isSelected
              ? 'border-green-dark bg-green-dark text-white'
              : 'border-dark-200 bg-white',
          )}
        >
          {!isAssigned && isSelected && <Check className="size-3" strokeWidth={3} />}
        </div>

        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
              <TreeDeciduous className="size-4 shrink-0 text-green-dark" aria-hidden />
              <span className="font-semibold text-sm truncate">{tree.species}</span>
            </div>
            <Badge variant="green-dark" size="lg" className="shrink-0 tabular-nums font-bold">
              <MapPin className="mr-1 size-3" aria-hidden />
              {formatDistance(distanceMeters)}
            </Badge>
          </div>

          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-dark-800">
            <span className="font-mono text-xs text-dark-600">{tree.number}</span>
            <span className="text-dark-200" aria-hidden>
              |
            </span>
            <span className="text-dark-600 text-xs">
              {tree.treeClusterId ? (clusterRes?.name ?? '…') : t('nearestTreeList.unassignedGroup')}
            </span>
            {isAssigned && (
              <>
                <span className="text-dark-200" aria-hidden>
                  |
                </span>
                <Badge variant="muted" size="default" aria-label={t('treeSearch.sensorAssignedBadge')}>
                  {t('treeSearch.sensorAssignedBadge')}
                </Badge>
              </>
            )}
          </div>
        </div>
      </div>
    </button>
  )
}

const NearestTreeList = ({ trees, selectedTreeId, onSelect }: NearestTreeListProps) => {
  const { t } = useTranslation('sensor')
  useEffect(() => {
    if (selectedTreeId !== null) return
    const firstSelectable = trees.find((entry) => entry.tree.sensor == null)
    if (firstSelectable) {
      onSelect(firstSelectable.tree.id)
    }
  }, [trees, selectedTreeId, onSelect])

  return (
    <section aria-label={t('nearestTreeList.title')}>
      <div className="flex items-center gap-2 mb-3">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-dark-600">
          {t('nearestTreeList.title')}
        </h2>
        <Badge variant="muted" size="default">
          {trees.length}
        </Badge>
      </div>

      <div
        className="flex flex-col gap-2"
        role="radiogroup"
        aria-label={t('nearestTreeList.selectAriaLabel')}
      >
        {trees.map((entry) => (
          <NearestTreeListItem
            key={entry.tree.id}
            entry={entry}
            isSelected={entry.tree.id === selectedTreeId}
            onSelect={() => onSelect(entry.tree.id)}
          />
        ))}
      </div>
    </section>
  )
}

export default NearestTreeList
export type { NearestTreeListProps }
