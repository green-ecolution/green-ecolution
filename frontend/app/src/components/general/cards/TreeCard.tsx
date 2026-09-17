import { FolderClosed } from 'lucide-react'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { Tree } from '@/api/backendApi'
import { Link } from '@tanstack/react-router'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { Badge, ListCard } from '@green-ecolution/ui'
import SensorIcon from '@/components/icons/Sensor'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { highlightMatch } from '@/lib/highlightMatch'

interface TreeCardProps {
  tree: Tree
  query?: string
}

const TreeCard: React.FC<TreeCardProps> = ({ tree, query = '' }) => {
  const { t } = useTranslation('tree')
  const getWateringStatusDetails = useWateringStatusDetails()
  const statusDetails = getWateringStatusDetails(tree.wateringStatus ?? WateringStatus.Unknown)

  return (
    <ListCard asChild>
      <Link to="/trees/$treeId" params={{ treeId: tree.id.toString() }}>
        <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
          <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
            <span className="font-lato text-lg font-bold tabular-nums">
              <span className="sr-only">{t('card.numberSrLabel')}</span>
              {highlightMatch(tree.number ?? t('card.numberUnknown'), query)}
            </span>
            <span className="text-base font-medium text-dark-800">
              {highlightMatch(tree.species, query)}
            </span>
          </div>
          <Badge variant={statusDetails.color} className="gap-2">
            <span
              aria-hidden
              className="size-2 rounded-full"
              style={{ backgroundColor: statusDetails.colorHex }}
            />
            {statusDetails.label}
          </Badge>
        </div>

        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-sm text-dark-600">
          <span className="flex items-center gap-2">
            <FolderClosed aria-hidden className="size-4" />
            <span className="sr-only">{t('card.clusterSrLabel')}</span>
            {tree.treeClusterName ?? t('card.clusterUnassigned')}
          </span>
          <span className="flex items-center gap-2">
            <SensorIcon aria-hidden className="size-4" />
            <span className="sr-only">{t('card.sensorSrLabel')}</span>
            {tree.sensor?.id ?? t('card.sensorUnassigned')}
          </span>
        </div>
      </Link>
    </ListCard>
  )
}

export default TreeCard
