import TreeIcon from '@/components/icons/Tree'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import type { TreeClusterInList } from '@/api/backendApi'
import { Link } from '@tanstack/react-router'
import { MapPin } from 'lucide-react'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { ListCard, ListCardStatus, ListCardTitle, ListCardMeta } from '@green-ecolution/ui'

interface TreeclusterCardProps {
  treecluster: TreeClusterInList
}

const TreeclusterCard: React.FC<TreeclusterCardProps> = ({ treecluster }) => {
  const { t } = useTranslation('treecluster')
  const getWateringStatusDetails = useWateringStatusDetails()
  const statusDetails = getWateringStatusDetails(treecluster.wateringStatus)

  return (
    <ListCard asChild columns="1fr 2fr 1.5fr 1fr">
      <Link
        to={`/treecluster/$treeclusterId`}
        params={{
          treeclusterId: treecluster.id.toString(),
        }}
      >
        <ListCardStatus status={statusDetails.color}>{statusDetails.label}</ListCardStatus>

        <ListCardTitle>{treecluster.name}</ListCardTitle>

        <ListCardMeta>
          <MapPin className="w-5 h-5" />
          <p>
            <span>{treecluster.address}, </span>
            <br />
            <span className="text-dark-600 lg:block lg:text-sm">
              {treecluster.region?.name ?? '-'}
            </span>
          </p>
        </ListCardMeta>

        <ListCardMeta>
          <TreeIcon className="w-5 h-5 mt-0.5" />
          <p>
            {t('listRow.treeCount', {
              count: treecluster.treeIds ? treecluster.treeIds.length : 0,
            })}
          </p>
        </ListCardMeta>
      </Link>
    </ListCard>
  )
}

export default TreeclusterCard
