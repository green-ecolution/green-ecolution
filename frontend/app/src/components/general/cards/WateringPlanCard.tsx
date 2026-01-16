import { getWateringPlanStatusDetails } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { WateringPlanInList } from '@green-ecolution/backend-client'
import { Link } from '@tanstack/react-router'
import React from 'react'
import {
  Badge,
  ListCard,
  ListCardTitle,
  ListCardDescription,
} from '@green-ecolution/ui'
import { format } from 'date-fns'
import { roundTo } from '@/lib/utils'

interface WateringPlanCardProps {
  wateringPlan: WateringPlanInList
}

const WateringPlanCard: React.FC<WateringPlanCardProps> = ({ wateringPlan }) => {
  const statusDetails = getWateringPlanStatusDetails(wateringPlan.status)
  const date = wateringPlan?.date
    ? format(new Date(wateringPlan?.date), 'dd.MM.yyyy')
    : 'Keine Angabe'

  return (
    <ListCard asChild columns="1.3fr 1.5fr 1fr 1.5fr 1.5fr" className="lg:py-10">
      <Link
        to={`/watering-plans/$wateringPlanId`}
        params={{
          wateringPlanId: wateringPlan.id.toString(),
        }}
      >
        <div>
          <Badge variant={statusDetails.color ?? 'outline-dark'} size="lg">{statusDetails.label}</Badge>
        </div>

        <div>
          <ListCardTitle className="mb-0.5">
            <span className="lg:sr-only">Einsatzplan: </span>
            {date}
          </ListCardTitle>
          <p className="text-dark-600 lg:text-sm">
            Fahrzeug:&nbsp;
            {wateringPlan.transporter.numberPlate}
            {wateringPlan.trailer && <span> | {wateringPlan.trailer.numberPlate}</span>}
          </p>
        </div>

        <ListCardDescription>
          <span className="lg:sr-only">Länge:&nbsp;</span>
          {`${roundTo(wateringPlan.distance, 2)} km`}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">Anzahl der Mitarbeitenden:&nbsp;</span>
          {wateringPlan.userIds.length} Mitarbeitende
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">Anzahl der Bewässerungsgruppen:&nbsp;</span>
          {wateringPlan.treeclusters.length}
          {wateringPlan.treeclusters.length === 1 ? ' Gruppe' : ' Gruppen'}
        </ListCardDescription>
      </Link>
    </ListCard>
  )
}

export default WateringPlanCard
