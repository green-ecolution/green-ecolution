import { getWateringPlanStatusDetails } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { WateringPlanInList } from '@green-ecolution/backend-client'
import { Link } from '@tanstack/react-router'
import React from 'react'
import Pill from '../Pill'
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
    <Link
      to={`/watering-plans/$wateringPlanId`}
      params={{
        wateringPlanId: wateringPlan.id.toString(),
      }}
      className="bg-white border border-dark-50 p-6 rounded-xl shadow-cards flex flex-col gap-y-4 transition-all ease-in-out duration-300 hover:bg-green-dark-50 hover:border-green-dark lg:grid lg:grid-cols-[1.3fr,1.5fr,1fr,1.5fr,1.5fr] lg:items-center lg:gap-5 lg:py-10 xl:px-10"
    >
      <Pill label={statusDetails.label} theme={statusDetails.color ?? 'grey'}></Pill>
      <div>
        <h2 className="text-dark font-bold text-lg mb-0.5">
          <span className="lg:sr-only">Einsatzplan: </span>
          {date}
        </h2>
        <p className="text-dark-600 lg:text-sm">
          Fahrzeug:&nbsp;
          {wateringPlan.transporter.numberPlate}
          {wateringPlan.trailer && <span> | {wateringPlan.trailer.numberPlate}</span>}
        </p>
      </div>

      <p className="text-dark-800">
        <span className="lg:sr-only">Länge:&nbsp;</span>
        {`${roundTo(wateringPlan.distance, 2)} km`}
      </p>

      <p className="text-dark-800">
        <span className="lg:sr-only">Anzahl der Mitarbeitenden:&nbsp;</span>
        {wateringPlan.userIds.length} Mitarbeitende
      </p>

      <p className="text-dark-800">
        <span className="lg:sr-only">Anzahl der Bewässerungsgruppen:&nbsp;</span>
        {wateringPlan.treeclusters.length}
        {wateringPlan.treeclusters.length === 1 ? ' Gruppe' : ' Gruppen'}
      </p>
    </Link>
  )
}

export default WateringPlanCard
