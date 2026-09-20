import { Droplets, IdCard, Truck } from 'lucide-react'
import type { Vehicle } from '@/api/backendApi'
import { Link } from '@tanstack/react-router'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { Badge, ListCard } from '@green-ecolution/ui'
import { useVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import { useVehicleTypeLabel } from '@/hooks/details/useDetailsForVehicleType'
import { highlightMatch } from '@/lib/highlightMatch'

interface VehicleCardProps {
  vehicle: Vehicle
  query?: string
}

const VehicleCard: React.FC<VehicleCardProps> = ({ vehicle, query = '' }) => {
  const { t } = useTranslation('vehicle')
  const getVehicleStatusDetails = useVehicleStatusDetails()
  const getVehicleTypeLabel = useVehicleTypeLabel()
  const statusDetails = getVehicleStatusDetails(vehicle.status)

  return (
    <ListCard asChild className="@container">
      <Link to="/vehicles/$vehicleId" params={{ vehicleId: vehicle.id.toString() }}>
        <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          <span className="flex items-center gap-2.5">
            <span
              aria-hidden
              title={statusDetails.label}
              className="size-2.5 shrink-0 rounded-full"
              style={{ backgroundColor: statusDetails.colorHex }}
            />
            <span className="font-lato text-lg font-bold tabular-nums">
              <span className="sr-only">
                {t('card.statusSrLabel')}
                {statusDetails.label}
              </span>
              <span className="sr-only">{t('card.numberPlateSrLabel')}</span>
              {highlightMatch(vehicle.numberPlate, query)}
            </span>
          </span>
          <span className="text-base font-medium text-dark-800">
            {highlightMatch(vehicle.model, query)}
          </span>
          {vehicle.archivedAt && <Badge variant="outline-dark">{t('card.archivedBadge')}</Badge>}
        </div>

        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-sm text-dark-600">
          <span className="flex items-center gap-2">
            <Truck aria-hidden className="size-4" />
            <span className="sr-only">{t('card.typeSrLabel')}</span>
            {getVehicleTypeLabel(vehicle.type)}
          </span>
          <span className="flex items-center gap-2">
            <Droplets aria-hidden className="size-4" />
            <span className="sr-only">{t('card.waterCapacitySrLabel')}</span>
            {t('card.waterCapacityValue', { value: vehicle.waterCapacity })}
          </span>
          {/* Secondary fact, dropped as the card narrows so the row never wraps. */}
          <span className="hidden items-center gap-2 @min-[30rem]:flex">
            <IdCard aria-hidden className="size-4" />
            <span className="sr-only">{t('card.drivingLicenseSrLabel')}</span>
            {vehicle.drivingLicense}
          </span>
        </div>
      </Link>
    </ListCard>
  )
}

export default VehicleCard
