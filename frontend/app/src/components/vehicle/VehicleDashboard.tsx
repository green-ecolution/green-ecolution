import EntityDetailHeader from '../general/EntityDetailHeader'
import { Badge } from '@green-ecolution/ui'
import { useVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import GeneralLink from '../general/links/GeneralLink'
import type { Vehicle } from '@/api/backendApi'
import { VehicleStatus } from '@green-ecolution/backend-client'
import { useVehicleTypeLabel } from '@/hooks/details/useDetailsForVehicleType'
import { DetailedList } from '@green-ecolution/ui'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useTranslation } from 'react-i18next'

interface VehicleDashboardProps {
  vehicle: Vehicle
}

const VehicleDashboard = ({ vehicle }: VehicleDashboardProps) => {
  const { t } = useTranslation(['vehicle', 'common'])
  const getVehicleStatusDetails = useVehicleStatusDetails()
  const getVehicleTypeLabel = useVehicleTypeLabel()
  const statusDetails = getVehicleStatusDetails(vehicle.status)
  const vehicleType = getVehicleTypeLabel(vehicle.type)
  const canEdit = useHasPermission(['vehicle:update'])
  const noData = t('common:state.noData')

  const vehicleData = [
    {
      label: t('detail.modelLabel'),
      value: vehicle?.model ?? noData,
    },
    {
      label: t('detail.typeLabel'),
      value: vehicleType ?? noData,
    },
    {
      label: t('detail.drivingLicenseLabel'),
      value: vehicle?.drivingLicense ?? noData,
    },
    {
      label: t('detail.heightLabel'),
      value: vehicle?.height ? t('detail.heightValue', { value: vehicle.height }) : noData,
    },
    {
      label: t('detail.widthLabel'),
      value: vehicle?.width ? t('detail.widthValue', { value: vehicle.width }) : noData,
    },
    {
      label: t('detail.numberPlateLabel'),
      value: vehicle?.numberPlate ?? noData,
    },
    {
      label: t('detail.waterCapacityLabel'),
      value: vehicle?.waterCapacity
        ? t('detail.waterCapacityValue', { value: vehicle.waterCapacity })
        : noData,
    },
    {
      label: t('detail.lengthLabel'),
      value: vehicle?.length ? t('detail.lengthValue', { value: vehicle.length }) : noData,
    },
    {
      label: t('detail.weightLabel'),
      value: vehicle?.weight ? t('detail.weightValue', { value: vehicle.weight }) : noData,
    },
  ]

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/vehicles' }, label: t('detail.backLabel') }}
        title={<>{t('detail.title', { numberPlate: vehicle.numberPlate })}</>}
        badge={
          <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
            {statusDetails?.label ?? noData}
          </Badge>
        }
        editLink={
          canEdit
            ? {
                label: t('detail.editLabel'),
                link: {
                  to: `/vehicles/$vehicleId/edit`,
                  params: { vehicleId: String(vehicle.id) },
                },
              }
            : undefined
        }
      >
        {vehicle.description && <p className="mb-4">{vehicle.description}</p>}
      </EntityDetailHeader>

      {vehicle.status == VehicleStatus.Active && (
        <div className="h-full shadow-cards flex flex-col gap-y-3 rounded-xl border border-green-light bg-green-light-50 p-6 mt-6">
          <div className="flex items-center justify-between">
            <p className="text-lg font-semibold">{t('detail.activeNotice')}</p>
            <GeneralLink
              label={t('detail.wateringPlansLinkLabel')}
              link={{
                to: '/watering-plans',
              }}
            />
          </div>
        </div>
      )}

      <section className="mt-16">
        <DetailedList headline={t('detail.headline')} details={vehicleData} />
      </section>
    </>
  )
}

export default VehicleDashboard
