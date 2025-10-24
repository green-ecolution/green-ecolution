import React from 'react'
import Pill from '../Pill'
import { StatusColor } from '@/hooks/details/useDetailsForWateringPlanStatus'

interface EntitiesStatusCard {
  statusDetails: { label: string; color: StatusColor; description: string }
  label: string
  hasPill?: boolean
}

const EntitiesStatusCard: React.FC<EntitiesStatusCard> = ({
  statusDetails,
  label,
  hasPill = false,
}) => {
  const backgroundColor =
    statusDetails.color === 'dark-400' || statusDetails.color === 'dark-600'
      ? 'dark-50'
      : `${statusDetails.color}-100`

  return (
    <div className={`h-full space-y-3 bg-${backgroundColor} rounded-xl p-6`}>
      <h2 className="text-sm text-dark-700 font-medium">{label}</h2>
      <p
        className={`relative font-bold text-xl ${hasPill ? '' : `pl-7 before:absolute before:w-4 before:h-4 before:rounded-full before:left-0 before:top-2 before:bg-${statusDetails.color}`}`}
      >
        {hasPill && <Pill label={statusDetails.label} theme={statusDetails.color ?? 'dark-400'} />}
        <span className={hasPill ? 'sr-only' : ''}>{statusDetails.label}</span>
      </p>
      <p className="text-sm">{statusDetails.description}</p>
    </div>
  )
}

export default EntitiesStatusCard
