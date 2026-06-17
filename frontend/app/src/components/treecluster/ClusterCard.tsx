import React from 'react'
import { Link } from '@tanstack/react-router'
import { ArrowRight, MapPin } from 'lucide-react'
import { Badge, Card } from '@green-ecolution/ui'
import TreeIcon from '@/components/icons/Tree'
import ClusterTreeDots from '@/components/treecluster/ClusterTreeDots'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { soilConditionLabel } from '@/hooks/details/useDetailsForSoilCondition'
import { SoilCondition } from '@/api/backendApi'
import type { TreeClusterInList } from '@/api/backendApi'

interface ClusterCardProps {
  treecluster: TreeClusterInList
  index?: number
}

const MS_PER_DAY = 1000 * 60 * 60 * 24

const lastWateredLabel = (lastWatered?: string | null): string => {
  if (!lastWatered) return 'noch nicht bewässert'

  const watered = new Date(lastWatered)
  if (Number.isNaN(watered.getTime())) return 'noch nicht bewässert'

  const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
  const days = Math.round((startOfDay(new Date()) - startOfDay(watered)) / MS_PER_DAY)

  if (days <= 0) return 'heute'
  if (days === 1) return 'gestern'
  return `vor ${days} Tagen`
}

const Metric: React.FC<{
  icon: React.ReactNode
  label: string
  value: React.ReactNode
}> = ({ icon, label, value }) => (
  <div className="flex flex-col gap-1">
    <span className="flex items-center gap-1.5 text-xs text-dark-600">
      {icon}
      {label}
    </span>
    <span className="font-lato font-semibold text-dark-900 truncate">{value}</span>
  </div>
)

const ClusterCard: React.FC<ClusterCardProps> = ({ treecluster, index = 0 }) => {
  const status = getWateringStatusDetails(treecluster.wateringStatus)
  const treeCount = treecluster.treeIds?.length ?? 0
  const hasSoil = treecluster.soilCondition && treecluster.soilCondition !== SoilCondition.Unknown
  const soilLabel = hasSoil ? soilConditionLabel(treecluster.soilCondition) : '–'

  return (
    <Card
      variant="outlined"
      style={{ animationDelay: `${Math.min(index, 12) * 60}ms`, opacity: 0 }}
      className="group relative flex h-full flex-col overflow-hidden animate-[fadeInUp_0.5s_ease-out_forwards] transition-shadow duration-300 hover:shadow-md focus-within:shadow-md"
    >
      <span
        aria-hidden
        className="absolute inset-y-0 left-0 w-1"
        style={{ backgroundColor: status.colorHex }}
      />

      <Link
        to="/treecluster/$treeclusterId"
        params={{ treeclusterId: treecluster.id.toString() }}
        className="flex flex-1 flex-col gap-5 p-6 pl-7 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:rounded-xl"
        aria-label={`Details zur Bewässerungsgruppe ${treecluster.name}`}
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
              {treecluster.region?.name ?? '–'}
            </span>
          </p>
        </div>

        <div className="grid grid-cols-3 gap-3 border-y border-border py-4">
          <Metric icon={<TreeIcon className="h-3.5 w-3.5" />} label="Bäume" value={treeCount} />
          <Metric
            icon={<span className="h-2 w-2 rounded-full bg-green-dark" aria-hidden />}
            label="Sensor-Bäume"
            value={treecluster.sensorCount}
          />
          <Metric
            icon={<MapPin className="h-3.5 w-3.5" aria-hidden />}
            label="Bodenart"
            value={soilLabel}
          />
        </div>

        <ClusterTreeDots treeCount={treeCount} sensorCount={treecluster.sensorCount} />

        <footer className="mt-auto flex items-center justify-between gap-3 pt-1 text-sm">
          <span className="min-w-0 truncate text-dark-600">
            Zuletzt bewässert: {lastWateredLabel(treecluster.lastWatered)}
          </span>
          <span className="flex shrink-0 items-center gap-1 font-semibold text-green-dark">
            Details
            <ArrowRight
              className="h-4 w-4 transition-transform duration-300 group-hover:translate-x-1"
              aria-hidden
            />
          </span>
        </footer>
      </Link>
    </Card>
  )
}

export default ClusterCard
