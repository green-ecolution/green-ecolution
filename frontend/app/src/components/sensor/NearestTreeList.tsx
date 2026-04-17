import { treeClusterIdQuery } from '@/api/queries'
import { type TreeWithDistance } from '@green-ecolution/backend-client'
import { Badge, cn } from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { Check, MapPin, TreeDeciduous } from 'lucide-react'
import { useEffect } from 'react'

function formatDistance(meters: number): string {
  if (meters >= 1000) {
    return `${new Intl.NumberFormat('de-DE', { maximumFractionDigits: 1 }).format(meters / 1000)} km`
  }
  if (meters < 10) {
    return `${new Intl.NumberFormat('de-DE', { maximumFractionDigits: 1 }).format(meters)} m`
  }
  return `${new Intl.NumberFormat('de-DE', { maximumFractionDigits: 0 }).format(meters)} m`
}

interface NearestTreeListProps {
  trees: TreeWithDistance[]
  selectedTreeId: number | null
  onSelect: (treeId: number) => void
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
  const { tree, distanceMeters } = entry

  const clusterId = tree.treeClusterId ? String(tree.treeClusterId) : null
  const { data: clusterRes } = useQuery({
    ...treeClusterIdQuery(clusterId!),
    enabled: clusterId !== null,
  })

  return (
    <button
      type="button"
      onClick={onSelect}
      aria-pressed={isSelected}
      className={cn(
        'relative w-full text-left rounded-xl border bg-white p-4 shadow-cards',
        'transition-all duration-200 ease-in-out',
        'hover:bg-green-dark-50/50',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-dark focus-visible:ring-offset-2',
        isSelected
          ? 'border-green-dark ring-2 ring-green-dark/20 bg-green-dark-50/30'
          : 'border-dark-100',
      )}
    >
      <div className="flex items-start gap-3">
        {/* Selection indicator */}
        <div
          className={cn(
            'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border-2 transition-colors',
            isSelected ? 'border-green-dark bg-green-dark text-white' : 'border-dark-200 bg-white',
          )}
        >
          {isSelected && <Check className="size-3" strokeWidth={3} />}
        </div>

        {/* Content */}
        <div className="min-w-0 flex-1">
          {/* Top row: species + distance */}
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

          {/* Details row */}
          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-dark-800">
            <span className="font-mono text-xs text-dark-600">{tree.number}</span>
            <span className="text-dark-200" aria-hidden>
              |
            </span>
            <span className="text-dark-600 text-xs">
              {tree.treeClusterId ? (clusterRes?.name ?? '…') : 'Nicht zugeordnet'}
            </span>
          </div>
        </div>
      </div>
    </button>
  )
}

const NearestTreeList = ({ trees, selectedTreeId, onSelect }: NearestTreeListProps) => {
  useEffect(() => {
    if (selectedTreeId === null && trees.length > 0) {
      onSelect(trees[0].tree.id)
    }
  }, [trees, selectedTreeId, onSelect])

  return (
    <section aria-label="Bäume in der Nähe">
      <div className="flex items-center gap-2 mb-3">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-dark-600">
          Bäume in der Nähe
        </h2>
        <Badge variant="muted" size="default">
          {trees.length}
        </Badge>
      </div>

      <div className="flex flex-col gap-2" role="radiogroup" aria-label="Baum auswählen">
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
