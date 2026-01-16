import React from 'react'
import { Badge } from '@green-ecolution/ui'
import { StatusColor } from '@/hooks/details/useDetailsForWateringPlanStatus'

interface EntitiesStatusCard {
  statusDetails: { label: string; color: StatusColor; description: string }
  label: string
  hasPill?: boolean
}

const backgroundColorMap: Partial<Record<StatusColor, string>> = {
  'outline-dark': 'bg-dark-50',
  'outline-red': 'bg-red-100',
  'outline-yellow': 'bg-yellow-100',
  'outline-green-dark': 'bg-green-dark-100',
  'outline-green-light': 'bg-green-light-100',
}

const dotColorMap: Partial<Record<StatusColor, string>> = {
  'outline-dark': 'bg-dark-400',
  'outline-red': 'bg-red',
  'outline-yellow': 'bg-yellow',
  'outline-green-dark': 'bg-green-dark',
  'outline-green-light': 'bg-green-light',
}

const EntitiesStatusCard: React.FC<EntitiesStatusCard> = ({
  statusDetails,
  label,
  hasPill = false,
}) => {
  const backgroundColor = backgroundColorMap[statusDetails.color] ?? 'bg-dark-50'
  const dotColor = dotColorMap[statusDetails.color] ?? 'bg-dark-400'

  return (
    <div className={`h-full space-y-3 rounded-xl p-6 ${backgroundColor}`}>
      <h2 className="text-sm text-dark-700 font-medium">{label}</h2>
      <p
        className={`relative font-bold text-xl ${hasPill ? '' : `pl-7 before:absolute before:w-4 before:h-4 before:rounded-full before:left-0 before:top-2 ${dotColor}`}`}
      >
        {hasPill && <Badge variant={statusDetails.color ?? 'outline-dark'} size="lg">{statusDetails.label}</Badge>}
        <span className={hasPill ? 'sr-only' : ''}>{statusDetails.label}</span>
      </p>
      <p className="text-sm">{statusDetails.description}</p>
    </div>
  )
}

export default EntitiesStatusCard
