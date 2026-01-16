import { treeClusterIdQuery } from '@/api/queries'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { WateringStatus, Tree } from '@green-ecolution/backend-client'
import { useQuery } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import React from 'react'
import {
  ListCard,
  ListCardStatus,
  ListCardTitle,
  ListCardDescription,
} from '@green-ecolution/ui'

interface TreeCardProps {
  tree: Tree
  showTreeClusterInfo?: boolean
}

const TreeCard: React.FC<TreeCardProps> = ({ tree, showTreeClusterInfo = true }) => {
  const clusterId = tree.treeClusterId ? String(tree.treeClusterId) : null
  const { data: clusterRes } = useQuery({
    ...treeClusterIdQuery(clusterId!),
    enabled: clusterId !== null,
  })
  const statusDetails = getWateringStatusDetails(
    tree.wateringStatus ?? WateringStatus.WateringStatusUnknown,
  )

  const columns = showTreeClusterInfo ? '1fr 1.5fr 1fr 1fr' : '1.5fr 2fr 1fr'

  return (
    <ListCard asChild columns={columns}>
      <Link
        to="/trees/$treeId"
        params={{
          treeId: tree.id.toString(),
        }}
      >
        <ListCardStatus status={statusDetails.color}>
          {statusDetails.label}
        </ListCardStatus>

        <ListCardTitle>{tree.species}</ListCardTitle>

        <ListCardDescription>
          <span className="lg:sr-only">Baumnummer: </span>
          {tree.number ?? 'Unbekannt'}
        </ListCardDescription>

        {showTreeClusterInfo && (
          <ListCardDescription>
            <span className="lg:sr-only">Bewässerungsgruppe: </span>
            {tree.treeClusterId ? (
              <span>{clusterRes?.name}</span>
            ) : (
              <span className="text-red">Nicht zugeordnet</span>
            )}
          </ListCardDescription>
        )}
      </Link>
    </ListCard>
  )
}

export default TreeCard
