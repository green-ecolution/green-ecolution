import { getVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import { Vehicle } from '@green-ecolution/backend-client'
import { Link } from '@tanstack/react-router'
import Pill from '@/components/general/Pill'
import React from 'react'
import { getVehicleType } from '@/hooks/details/useDetailsForVehicleType'

interface VehicleCard {
  vehicle: Vehicle
}

const VehicleCard: React.FC<VehicleCard> = ({ vehicle }) => {
  const statusDetails = getVehicleStatusDetails(vehicle.status)
  const vehicleType = getVehicleType(vehicle.type)

  return (
    <Link
      to={`/vehicles/$vehicleId`}
      params={{
        vehicleId: vehicle.id.toString(),
      }}
      className="bg-white border border-dark-50 p-6 rounded-xl shadow-cards flex flex-col gap-y-4 transition-all ease-in-out duration-300 hover:bg-green-dark-50 hover:border-green-dark lg:grid lg:grid-cols-5 lg:items-center lg:gap-5 lg:py-10 xl:px-10"
    >
      <Pill
        label={statusDetails?.label ?? 'Keine Angabe'}
        theme={statusDetails?.color ?? 'dark-400'}
      />
      <div>
        <h2 className="text-dark font-bold text-lg mb-0.5">
          <span className="lg:sr-only">Kennzeichen: </span>
          {vehicle.numberPlate}
        </h2>
        <p className="text-dark-600 lg:text-sm">{vehicleType}</p>
      </div>

      <p className="text-dark-800">
        <span className="lg:sr-only">Wasserkapazität:&nbsp;</span>
        {vehicle.waterCapacity} Liter
      </p>

      <p className="text-dark-800">
        <span className="lg:sr-only">Modell:&nbsp;</span>
        {vehicle.model}
      </p>

      <p className="text-dark-800">
        <span className="lg:sr-only">Benötigte Führerscheinklasse:&nbsp;</span>
        {vehicle.drivingLicense}
      </p>
    </Link>
  )
}

export default VehicleCard
