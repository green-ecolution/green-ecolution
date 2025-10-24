import React from 'react'
import SelectedCardTree from './selected_card/SelectedCardTree'
import SelectedCardCluster from './selected_card/SelectedCardTreeCluster'

export interface SelectedCardProps {
  id: number
  type: 'tree' | 'cluster'
  onClick?: (id: number) => void
}

const SelectedCard: React.FC<SelectedCardProps> = ({ onClick, id, type }) => {
  if (type === 'tree') {
    return <SelectedCardTree onClick={onClick} id={id} />
  } else {
    return <SelectedCardCluster onClick={onClick} id={id} />
  }
}

export default SelectedCard
