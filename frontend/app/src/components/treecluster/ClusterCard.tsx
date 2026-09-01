import React from 'react'
import { Link } from '@tanstack/react-router'
import { ArrowRight, Layers, MapPin } from 'lucide-react'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { Badge, Card } from '@green-ecolution/ui'
import TreeIcon from '@/components/icons/Tree'
import ClusterTreeDots from '@/components/treecluster/ClusterTreeDots'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useSoilConditionLabel } from '@/hooks/details/useDetailsForSoilCondition'
import { SoilCondition } from '@/api/backendApi'
import type { TreeClusterInList } from '@/api/backendApi'

interface ClusterCardProps {
  treecluster: TreeClusterInList
}

const MS_PER_DAY = 1000 * 60 * 60 * 24

const lastWateredLabel = (
  lastWatered: string | null | undefined,
  t: TFunction<'treecluster'>,
): string => {
  if (!lastWatered) return t('card.lastWateredNever')

  const watered = new Date(lastWatered)
  if (Number.isNaN(watered.getTime())) return t('card.lastWateredNever')

  const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
  const days = Math.round((startOfDay(new Date()) - startOfDay(watered)) / MS_PER_DAY)

  if (days <= 0) return t('card.lastWateredToday')
  if (days === 1) return t('card.lastWateredYesterday')
  return t('card.lastWateredDaysAgo', { count: days })
}

const Metric: React.FC<{
  icon: React.ReactNode
  label: string
  value: React.ReactNode
}> = ({ icon, label, value }) => (
  <div className="flex min-w-0 flex-col gap-1">
    <span className="flex items-center gap-1.5 whitespace-nowrap text-xs text-dark-600">
      {icon}
      {label}
    </span>
    <span className="font-lato font-semibold text-dark-900">{value}</span>
  </div>
)

const ClusterCard: React.FC<ClusterCardProps> = ({ treecluster }) => {
  const { t } = useTranslation('treecluster')
  const getWateringStatusDetails = useWateringStatusDetails()
  const soilConditionLabel = useSoilConditionLabel()
  const status = getWateringStatusDetails(treecluster.wateringStatus)
  const treeCount = treecluster.treeIds?.length ?? 0
  const hasSoil = treecluster.soilCondition && treecluster.soilCondition !== SoilCondition.Unknown
  const soilLabel = hasSoil ? soilConditionLabel(treecluster.soilCondition) : t('card.noSoilData')

  return (
    <Card
      variant="outlined"
      className="group flex h-full flex-col transition-shadow duration-base ease-out hover:shadow-md focus-within:shadow-md"
    >
      <Link
        to="/treecluster/$treeclusterId"
        params={{ treeclusterId: treecluster.id.toString() }}
        className="flex flex-1 flex-col gap-5 p-6 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:rounded-xl"
        aria-label={t('card.ariaLabelDetail', { name: treecluster.name })}
      >
        <header className="flex items-start justify-between gap-3">
          <div className="flex min-w-0 items-center gap-2">
            <span
              aria-hidden
              className="h-2.5 w-2.5 shrink-0 rounded-full"
              style={{ backgroundColor: status.colorHex }}
            />
            <h2 className="font-lato text-lg font-bold leading-tight text-dark-900 truncate">
              {treecluster.name}
            </h2>
          </div>
          <Badge variant={status.color} className="shrink-0">
            {status.label}
          </Badge>
        </header>

        <div className="flex items-start gap-2 text-sm">
          <MapPin className="mt-0.5 h-4 w-4 shrink-0 text-dark-600" aria-hidden />
          <p className="min-w-0">
            <span className="block truncate text-dark-900">{treecluster.address}</span>
            <span className="block truncate text-xs text-dark-600">
              {treecluster.region?.name ?? t('card.noRegion')}
            </span>
          </p>
        </div>

        <div className="flex flex-col gap-3 border-y border-border py-4">
          <div className="grid grid-cols-2 gap-3">
            <Metric
              icon={<TreeIcon className="h-3.5 w-3.5" />}
              label={t('card.treesLabel')}
              value={treeCount}
            />
            <Metric
              icon={<span className="h-2 w-2 shrink-0 rounded-full bg-green-dark" aria-hidden />}
              label={t('card.sensorTreesLabel')}
              value={treecluster.sensorCount}
            />
          </div>
          <Metric
            icon={<Layers className="h-3.5 w-3.5 shrink-0" aria-hidden />}
            label={t('card.soilLabel')}
            value={soilLabel}
          />
        </div>

        <ClusterTreeDots treeCount={treeCount} sensorCount={treecluster.sensorCount} />

        <footer className="mt-auto flex items-center justify-between gap-3 pt-1 text-sm">
          <span className="min-w-0 truncate text-dark-600">
            {t('card.lastWateredPrefix', { label: lastWateredLabel(treecluster.lastWatered, t) })}
          </span>
          <span className="flex shrink-0 items-center gap-1 font-semibold text-green-dark">
            {t('card.detailsLabel')}
            <ArrowRight
              className="h-4 w-4 transition-transform duration-base ease-emphasized group-hover:translate-x-1 motion-reduce:transition-none"
              aria-hidden
            />
          </span>
        </footer>
      </Link>
    </Card>
  )
}

export default ClusterCard
