import { Tree } from '@green-ecolution/backend-client'
import { useSuspenseQuery } from '@tanstack/react-query'
import { treeQuery } from '@/api/queries'
import { TreeMarkerIcon } from '../MapMarker'
import MarkerList from './MarkerList'
import { getStatusColor } from '../utils'

const defaultSelectedTrees: number[] = []

export interface WithAllTreesProps {
  onClick?: (tree: Tree) => void
  selectedTrees: number[]
  hasHighlightedTree?: number
}

const WithAllTrees = ({
  onClick,
  selectedTrees = defaultSelectedTrees,
  hasHighlightedTree,
}: WithAllTreesProps) => {
  const { data } = useSuspenseQuery(treeQuery())

  const defineIcon = (t: Tree) => {
    return TreeMarkerIcon(
      getStatusColor(t.wateringStatus),
      selectedTrees?.includes(t.id) ?? false,
      hasHighlightedTree === t.id,
    )
  }

  return (
    <MarkerList
      data={data.data}
      onClick={onClick}
      icon={defineIcon}
      tooltipContent={(t) => t.number}
      tooltipOptions={{
        direction: 'top',
        offset: [5, -40],
        className: 'font-nunito-sans font-semibold',
      }}
    />
  )
}

export default WithAllTrees
