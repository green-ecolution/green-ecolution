import React from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { useNavigate, useSearch } from '@tanstack/react-router'
import { X } from 'lucide-react'
import { clusterStatisticsQuery } from '@/api/queries'
import { WateringStatus } from '@/api/backendApi'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'

const STATUSES = [
  WateringStatus.Bad,
  WateringStatus.Moderate,
  WateringStatus.Good,
  WateringStatus.JustWatered,
  WateringStatus.Unknown,
] as const

const ALWAYS_SHOWN: WateringStatus[] = [
  WateringStatus.Bad,
  WateringStatus.Moderate,
  WateringStatus.Good,
]

const ClusterStatusChips: React.FC<{ className?: string }> = ({ className }) => {
  const { data: stats } = useSuspenseQuery(clusterStatisticsQuery())
  const search = useSearch({ strict: false })
  const navigate = useNavigate()

  const active =
    'wateringStatuses' in search
      ? ((search.wateringStatuses as WateringStatus[] | undefined) ?? [])
      : []

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

  const visible = STATUSES.filter((s) => ALWAYS_SHOWN.includes(s) || countFor(s) > 0)

  return (
    <div
      role="group"
      aria-label="Nach Bewässerungszustand filtern"
      className={`flex items-center gap-2 overflow-x-auto pb-1 sm:flex-wrap sm:overflow-visible sm:pb-0 ${className ?? ''}`}
    >
      {visible.map((status) => {
        const details = getWateringStatusDetails(status)
        const isActive = active.includes(status)
        return (
          <button
            key={status}
            type="button"
            aria-pressed={isActive}
            onClick={() => toggle(status)}
            className={[
              'inline-flex shrink-0 cursor-pointer items-center gap-2 rounded-full border px-3 py-1.5 text-sm transition',
              'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1',
              isActive
                ? 'border-green-dark bg-dark-50 font-medium text-dark-900'
                : 'border-dark-200 bg-white text-dark-700 hover:border-green-dark hover:bg-dark-50',
            ].join(' ')}
          >
            <span
              className="h-2.5 w-2.5 shrink-0 rounded-full"
              style={{ backgroundColor: details.colorHex }}
              aria-hidden
            />
            <span className="font-semibold tabular-nums">{countFor(status)}</span>
            <span className="text-dark-600">{details.label}</span>
            {isActive && <X className="h-3.5 w-3.5 shrink-0 text-dark-500" aria-hidden />}
          </button>
        )
      })}
    </div>
  )
}

export default ClusterStatusChips
