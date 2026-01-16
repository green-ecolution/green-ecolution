import { getVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import { Vehicle } from '@green-ecolution/backend-client'
import { Link } from '@tanstack/react-router'
import {
  Badge,
  ListCard,
  ListCardTitle,
  ListCardDescription,
} from '@green-ecolution/ui'
import React from 'react'
import { getVehicleType } from '@/hooks/details/useDetailsForVehicleType'

interface VehicleCard {
  vehicle: Vehicle
}

const VehicleCard: React.FC<VehicleCard> = ({ vehicle }) => {
  const statusDetails = getVehicleStatusDetails(vehicle.status)
  const vehicleType = getVehicleType(vehicle.type)

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
            {statusDetails?.label ?? 'Keine Angabe'}
          </Badge>
        </div>

        <div>
          <ListCardTitle className="mb-0.5">
            <span className="lg:sr-only">Kennzeichen: </span>
            {vehicle.numberPlate}
          </ListCardTitle>
          <p className="text-dark-600 lg:text-sm">{vehicleType}</p>
        </div>

        <ListCardDescription>
          <span className="lg:sr-only">Wasserkapazität:&nbsp;</span>
          {vehicle.waterCapacity} Liter
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">Modell:&nbsp;</span>
          {vehicle.model}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">Benötigte Führerscheinklasse:&nbsp;</span>
          {vehicle.drivingLicense}
        </ListCardDescription>
      </Link>
    </ListCard>
  )
}

export default VehicleCard
