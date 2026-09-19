import { Droplets, FolderClosed, Sprout } from 'lucide-react'
import { format } from 'date-fns'
import { WateringStatus } from '@green-ecolution/backend-client'
import type { Tree } from '@/api/backendApi'
import { Link } from '@tanstack/react-router'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { ListCard } from '@green-ecolution/ui'
import SensorIcon from '@/components/icons/Sensor'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { highlightMatch } from '@/lib/highlightMatch'

interface TreeCardProps {
  tree: Tree
  query?: string
}

const TreeCard: React.FC<TreeCardProps> = ({ tree, query = '' }) => {
  const { t } = useTranslation('tree')
  const getWateringStatusDetails = useWateringStatusDetails()
  const dateLocale = useDateLocale()
  const statusDetails = getWateringStatusDetails(tree.wateringStatus ?? WateringStatus.Unknown)

  return (
    <ListCard asChild className="@container">
      <Link to="/trees/$treeId" params={{ treeId: tree.id.toString() }}>
        <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          <span className="flex items-center gap-2.5">
            <span
              aria-hidden
              title={statusDetails.label}
              className="size-2.5 shrink-0 rounded-full"
              style={{ backgroundColor: statusDetails.colorHex }}
            />
            <span className="font-lato text-lg font-bold tabular-nums">
              <span className="sr-only">
                {t('card.statusSrLabel')}
                {statusDetails.label}
              </span>
              <span className="sr-only">{t('card.numberSrLabel')}</span>
              {highlightMatch(tree.number ?? t('card.numberUnknown'), query)}
            </span>
          </span>
          <span className="text-base font-medium text-dark-800">
            {highlightMatch(tree.species, query)}
          </span>
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
          {/* Secondary facts, dropped as the card narrows so the row never wraps. */}
          <span className="hidden items-center gap-2 @min-[30rem]:flex">
            <Sprout aria-hidden className="size-4" />
            <span className="sr-only">{t('card.plantingYearSrLabel')}</span>
            <span className="tabular-nums">{tree.plantingYear}</span>
          </span>
          <span className="hidden items-center gap-2 @min-[42rem]:flex">
            <Droplets aria-hidden className="size-4" />
            <span className="sr-only">{t('card.lastWateredSrLabel')}</span>
            {tree.lastWatered
              ? format(new Date(tree.lastWatered), 'dd.MM.yyyy', { locale: dateLocale })
              : t('card.lastWateredNever')}
          </span>
        </div>
      </Link>
    </ListCard>
  )
}

export default TreeCard
