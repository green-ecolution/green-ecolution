import { useVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import type { Vehicle } from '@/api/backendApi'
import { Link } from '@tanstack/react-router'
import { Badge, ListCard, ListCardTitle, ListCardDescription } from '@green-ecolution/ui'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { useVehicleTypeLabel } from '@/hooks/details/useDetailsForVehicleType'

interface VehicleCard {
  vehicle: Vehicle
}

const VehicleCard: React.FC<VehicleCard> = ({ vehicle }) => {
  const { t } = useTranslation(['vehicle', 'common'])
  const getVehicleStatusDetails = useVehicleStatusDetails()
  const getVehicleTypeLabel = useVehicleTypeLabel()
  const statusDetails = getVehicleStatusDetails(vehicle.status)
  const vehicleType = getVehicleTypeLabel(vehicle.type)

  return (
    <ListCard asChild columns="repeat(5, 1fr)" className="lg:py-10">
      <Link
        to={`/vehicles/$vehicleId`}
        params={{
          vehicleId: vehicle.id.toString(),
        }}
      >
        <div>
          <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
            {statusDetails?.label ?? t('common:state.noData')}
          </Badge>
        </div>

        <div>
          <ListCardTitle className="mb-0.5">
            <span className="lg:sr-only">{t('card.numberPlateSrLabel')} </span>
            {vehicle.numberPlate}
          </ListCardTitle>
          <p className="text-dark-600 lg:text-sm">{vehicleType}</p>
        </div>

        <ListCardDescription>
          <span className="lg:sr-only">{t('card.waterCapacitySrLabel')}&nbsp;</span>
          {t('card.waterCapacityValue', { value: vehicle.waterCapacity })}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">{t('card.modelSrLabel')}&nbsp;</span>
          {vehicle.model}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">{t('card.drivingLicenseSrLabel')}&nbsp;</span>
          {vehicle.drivingLicense}
        </ListCardDescription>
      </Link>
    </ListCard>
  )
}

export default VehicleCard
