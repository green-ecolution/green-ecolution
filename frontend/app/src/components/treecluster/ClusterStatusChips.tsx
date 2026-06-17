import React from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { Badge } from '@green-ecolution/ui'
import { clusterStatisticsQuery } from '@/api/queries'
import { WateringStatus } from '@/api/backendApi'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

const PRIMARY_STATUSES = [WateringStatus.Bad, WateringStatus.Moderate, WateringStatus.Good] as const

const ClusterStatusChips: React.FC = () => {
  const { data: stats } = useSuspenseQuery(clusterStatisticsQuery())
  const search = useSearch({ strict: false })
  const navigate = useNavigate()

  const wateringStatuses =
    'wateringStatuses' in search
      ? (search.wateringStatuses as WateringStatus[] | undefined)
      : undefined
  const active = wateringStatuses ?? []

  const toggle = (status: WateringStatus) => {
    const next = active.includes(status) ? active.filter((s) => s !== status) : [...active, status]
    navigate({
      to: '/treecluster',
      search: (prev: Record<string, unknown>) => ({
        ...prev,
        wateringStatuses: next.length ? next : undefined,
        page: 1,
      }),
    }).catch((error) => console.error('Navigation failed:', error))
  }

  const countFor = (status: WateringStatus): number => {
    switch (status) {
      case WateringStatus.Bad:
        return stats.bad
      case WateringStatus.Moderate:
        return stats.moderate
      case WateringStatus.Good:
        return stats.good
      case WateringStatus.JustWatered:
        return stats.justWatered
      case WateringStatus.Unknown:
        return stats.unknown
    }
  }

  return (
    <div className="flex flex-wrap items-center gap-2 sm:gap-3">
      <span className="text-sm text-dark-600">
        {stats.total} Gruppen · {stats.trees} Bäume
      </span>

      {PRIMARY_STATUSES.map((status) => {
        const details = getWateringStatusDetails(status)
        const isActive = active.includes(status)
        return (
          <button
            key={status}
            type="button"
            aria-pressed={isActive}
            onClick={() => toggle(status)}
            className="cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 rounded-full"
          >
            <Badge variant={isActive ? details.color : 'muted'}>
              {countFor(status)} {details.label}
            </Badge>
          </button>
        )
      })}

      {stats.justWatered > 0 && (
        <Badge variant="muted">
          {stats.justWatered} {getWateringStatusDetails(WateringStatus.JustWatered).label}
        </Badge>
      )}

      {stats.unknown > 0 && (
        <Badge variant="muted">
          {stats.unknown} {getWateringStatusDetails(WateringStatus.Unknown).label}
        </Badge>
      )}
    </div>
  )
}

export default ClusterStatusChips
