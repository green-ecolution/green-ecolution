import { Tree } from '@green-ecolution/backend-client'
import { useSuspenseQuery } from '@tanstack/react-query'
import { treeQuery } from '@/api/queries'
import { TreeMarkerIcon } from '../MapMarker'
import MarkerList from './MarkerList'
import { getStatusColor } from '../utils'
import { memo, useCallback, useMemo } from 'react'

const defaultSelectedTrees: number[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

export interface WithAllTreesProps {
  onClick?: (tree: Tree) => void
  selectedTrees?: number[]
  hasHighlightedTree?: number
}

const WithAllTrees = memo(
  ({ onClick, selectedTrees = defaultSelectedTrees, hasHighlightedTree }: WithAllTreesProps) => {
    const { data } = useSuspenseQuery(treeQuery())

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
        data={data.data}
        onClick={onClick}
        icon={getIcon}
        getId={getId}
        tooltipContent={getTooltip}
        tooltipOptions={tooltipOptions}
      />
    )
  },
)

WithAllTrees.displayName = 'WithAllTrees'

export default WithAllTrees
