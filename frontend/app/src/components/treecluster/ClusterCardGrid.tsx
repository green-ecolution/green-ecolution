import React from 'react'
import ClusterCard from '@/components/treecluster/ClusterCard'
import type { TreeClusterInList } from '@/api/backendApi'

interface ClusterCardGridProps {
  data: TreeClusterInList[]
}

const ClusterCardGrid: React.FC<ClusterCardGridProps> = ({ data }) => {
  if (!data || data.length === 0) {
    return (
      <p className="mt-10 text-center text-dark-600">
        Es wurden leider keine Bewässerungsgruppen gefunden.
      </p>
    )
  }

  return (
    <ul className="grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-3">
      {data.map((cluster, index) => (
        <li key={cluster.id} className="h-full">
          <ClusterCard treecluster={cluster} index={index} />
        </li>
      ))}
    </ul>
  )
}

export default ClusterCardGrid
