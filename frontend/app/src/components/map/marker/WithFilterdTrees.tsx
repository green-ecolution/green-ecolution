import { Tree } from '@green-ecolution/backend-client'
import MarkerList from './MarkerList'
import { TreeMarkerIcon } from '../MapMarker'
import { getStatusColor } from '../utils'
import { memo, useCallback, useDeferredValue, useMemo } from 'react'

const defaultSelectedTrees: number[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

interface WithFilterdTreesProps {
  onClick?: (tree: Tree) => void
  selectedTrees?: number[]
  hasHighlightedTree?: number
  filterdTrees: Tree[]
}

const WithFilterdTrees = memo(
  ({
    onClick,
    selectedTrees = defaultSelectedTrees,
    hasHighlightedTree,
    filterdTrees,
  }: WithFilterdTreesProps) => {
    const deferredTrees = useDeferredValue(filterdTrees)
    const selectedSet = useMemo(() => new Set(selectedTrees), [selectedTrees])

    const getIcon = useCallback(
      (t: Tree) =>
        TreeMarkerIcon(
          getStatusColor(t.wateringStatus),
          selectedSet.has(t.id),
          hasHighlightedTree === t.id,
        ),
      [selectedSet, hasHighlightedTree],
    )

    const getId = useCallback((t: Tree) => t.id, [])
    const getTooltip = useCallback((t: Tree) => t.number, [])

    return (
      <MarkerList
        data={deferredTrees}
        onClick={onClick}
        icon={getIcon}
        getId={getId}
        tooltipContent={getTooltip}
        tooltipOptions={tooltipOptions}
      />
    )
  },
)

WithFilterdTrees.displayName = 'WithFilterdTrees'

export default WithFilterdTrees
