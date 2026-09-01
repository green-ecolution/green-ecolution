import { clusterQueries } from '@/api/queries'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { Tree } from '@/api/backendApi'
import { useQuery } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { ListCard, ListCardStatus, ListCardTitle, ListCardDescription } from '@green-ecolution/ui'

interface TreeCardProps {
  tree: Tree
  showTreeClusterInfo?: boolean
}

const TreeCard: React.FC<TreeCardProps> = ({ tree, showTreeClusterInfo = true }) => {
  const { t } = useTranslation('tree')
  const clusterId = tree.treeClusterId ? String(tree.treeClusterId) : null
  const { data: clusterRes } = useQuery({
    ...clusterQueries.detail(clusterId!),
    enabled: clusterId !== null,
  })
  const getWateringStatusDetails = useWateringStatusDetails()
  const statusDetails = getWateringStatusDetails(tree.wateringStatus ?? WateringStatus.Unknown)

  const columns = showTreeClusterInfo ? '1fr 1.5fr 1fr 1fr' : '1.5fr 2fr 1fr'

  return (
    <ListCard asChild columns={columns}>
      <Link
        to="/trees/$treeId"
        params={{
          treeId: tree.id.toString(),
        }}
      >
        <ListCardStatus status={statusDetails.color}>{statusDetails.label}</ListCardStatus>

        <ListCardTitle>{tree.species}</ListCardTitle>

        <ListCardDescription>
          <span className="lg:sr-only">{t('card.numberSrLabel')}</span>
          {tree.number ?? t('card.numberUnknown')}
        </ListCardDescription>

        {showTreeClusterInfo && (
          <ListCardDescription>
            <span className="lg:sr-only">{t('card.clusterSrLabel')}</span>
            {tree.treeClusterId ? (
              <span>{clusterRes?.name}</span>
            ) : (
              <span className="text-destructive">{t('card.clusterUnassigned')}</span>
            )}
          </ListCardDescription>
        )}
      </Link>
    </ListCard>
  )
}

export default TreeCard
