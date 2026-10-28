import { useState } from 'react'
import { Link } from '@tanstack/react-router'
import { useMutation } from '@tanstack/react-query'
import { ArrowRightLeft, ChevronDown, Pencil } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Badge,
  Button,
  ButtonGroup,
  DetailedList,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'
import { VehicleStatus } from '@green-ecolution/backend-client'
import EntityDetailHeader from '../general/EntityDetailHeader'
import GeneralLink from '../general/links/GeneralLink'
import TransferOrganizationDialog from '../general/transfer/TransferOrganizationDialog'
import { useTransferTargets } from '../general/transfer/useTransferTargets'
import { vehicleApi, type Vehicle } from '@/api/backendApi'
import { queryRoots } from '@/api/queries'
import createToast from '@/hooks/createToast'
import { useVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import { useVehicleTypeLabel } from '@/hooks/details/useDetailsForVehicleType'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'

interface VehicleDashboardProps {
  vehicle: Vehicle
}

const VehicleDashboard = ({ vehicle }: VehicleDashboardProps) => {
  const { t } = useTranslation(['vehicle', 'common'])
  const getVehicleStatusDetails = useVehicleStatusDetails()
  const getVehicleTypeLabel = useVehicleTypeLabel()
  const statusDetails = getVehicleStatusDetails(vehicle.status)
  const vehicleType = getVehicleTypeLabel(vehicle.type)
  const invalidate = useInvalidateQueryRoots()
  const showToast = createToast()
  const [transferOpen, setTransferOpen] = useState(false)
  const canEdit = useHasPermission(['vehicle:update'])
  const transferTargets = useTransferTargets('vehicle:update', vehicle.organizationId)
  const canTransfer = canEdit && transferTargets.length > 0
  const noData = t('common:state.noData')

  const transfer = useMutation({
    mutationFn: (organizationId: string) =>
      vehicleApi.transferVehicle({ vehicleId: vehicle.id, transferRequest: { organizationId } }),
    onSuccess: async () => {
      setTransferOpen(false)
      showToast(t('detail.transferSuccessToast'))
      await invalidate(queryRoots.vehicle, { reloadRoutes: true })
    },
    onError: () => showToast(t('detail.transferErrorToast'), 'error'),
  })

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
    ...(vehicle.capacities.water !== undefined
      ? [
          {
            label: t('detail.waterCapacityLabel'),
            value: t('detail.waterCapacityValue', { value: vehicle.capacities.water }),
          },
        ]
      : []),
    ...(vehicle.capacities.cargo_volume !== undefined
      ? [
          {
            label: t('detail.cargoVolumeLabel'),
            value: t('detail.cargoVolumeValue', { value: vehicle.capacities.cargo_volume }),
          },
        ]
      : []),
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
        actions={
          canEdit && (
            <ButtonGroup>
              <Button variant="outline" asChild>
                <Link to="/vehicles/$vehicleId/edit" params={{ vehicleId: String(vehicle.id) }}>
                  {t('detail.editLabel')}
                  <Pencil className="stroke-1" />
                </Link>
              </Button>
              {canTransfer && (
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      aria-label={t('detail.moreActionsAriaLabel')}
                      className="[&_svg]:size-4 [&_svg]:transition-transform [&_svg]:duration-base data-[state=open]:[&_svg]:rotate-180 motion-reduce:[&_svg]:transition-none"
                    >
                      <ChevronDown />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="min-w-[12rem]">
                    <DropdownMenuItem
                      className="gap-2 px-3 py-2"
                      onSelect={() => setTransferOpen(true)}
                    >
                      <ArrowRightLeft />
                      {t('common:transfer.menuItem')}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
            </ButtonGroup>
          )
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
                to: '/plans',
              }}
            />
          </div>
        </div>
      )}

      <section className="mt-16">
        <DetailedList headline={t('detail.headline')} details={vehicleData} />
      </section>

      <TransferOrganizationDialog
        open={transferOpen}
        onOpenChange={setTransferOpen}
        title={t('detail.transferDialogTitle', { numberPlate: vehicle.numberPlate })}
        description={t('detail.transferDialogDescription')}
        organizations={transferTargets}
        pending={transfer.isPending}
        onConfirm={(organizationId) => transfer.mutate(organizationId)}
      />
    </>
  )
}

export default VehicleDashboard
