import EntityDetailHeader from '../general/EntityDetailHeader'
import { Badge } from '@green-ecolution/ui'
import { getVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import GeneralLink from '../general/links/GeneralLink'
import type { Vehicle } from '@/api/backendApi'
import { VehicleStatus } from '@green-ecolution/backend-client'
import { getVehicleType } from '@/hooks/details/useDetailsForVehicleType'
import { DetailedList } from '@green-ecolution/ui'

interface VehicleDashboardProps {
  vehicle: Vehicle
}

const VehicleDashboard = ({ vehicle }: VehicleDashboardProps) => {
  const statusDetails = getVehicleStatusDetails(vehicle.status)
  const vehicleType = getVehicleType(vehicle.type)

  const vehicleData = [
    {
      label: 'Modell',
      value: vehicle?.model ?? 'Keine Angabe',
    },
    {
      label: 'Fahrzeug-Typ',
      value: vehicleType ?? 'Keine Angabe',
    },
    {
      label: 'Benötigte Führerscheinklasse',
      value: vehicle?.drivingLicense ?? 'Keine Angabe',
    },
    {
      label: 'Höhe des Fahrzeugs',
      value: vehicle?.height ? `${vehicle.height} Meter` : 'Keine Angabe',
    },
    {
      label: 'Breite des Fahrzeugs',
      value: vehicle?.width ? `${vehicle.width} Meter` : 'Keine Angabe',
    },
    {
      label: 'Nummernschild',
      value: vehicle?.numberPlate ?? 'Keine Angabe',
    },
    {
      label: 'Wasserkapazität',
      value: vehicle?.waterCapacity ? `${vehicle.waterCapacity} Liter` : 'Keine Angabe',
    },
    {
      label: 'Länge des Fahrzeugs',
      value: vehicle?.length ? `${vehicle.length} Meter` : 'Keine Angabe',
    },
    {
      label: 'Gewicht des Fahrzeugs',
      value: vehicle?.weight ? `${vehicle.weight} Tonnen` : 'Keine Angabe',
    },
  ]

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/vehicles' }, label: 'Alle Fahrzeuge' }}
        title={<>Fahrzeug: {vehicle.numberPlate}</>}
        badge={
          <Badge variant={statusDetails?.color ?? 'outline-dark'} size="lg">
            {statusDetails?.label ?? 'Keine Angabe'}
          </Badge>
        }
        editLink={{
          label: 'Fahrzeug bearbeiten',
          link: {
            to: `/vehicles/$vehicleId/edit`,
            params: { vehicleId: String(vehicle.id) },
          },
        }}
      >
        {vehicle.description && <p className="mb-4">{vehicle.description}</p>}
      </EntityDetailHeader>

      {vehicle.status == VehicleStatus.Active && (
        <div className="h-full shadow-cards flex flex-col gap-y-3 rounded-xl border border-green-light bg-green-light-50 p-6 mt-6">
          <div className="flex items-center justify-between">
            <p className="text-lg font-semibold">Dieses Fahrzeug befindet sich im Einsatz.</p>
            <GeneralLink
              label="Zu den Einsatzplänen"
              link={{
                to: '/watering-plans',
              }}
            />
          </div>
        </div>
      )}

      <section className="mt-16">
        <DetailedList headline="Daten zum Fahrzeug" details={vehicleData} />
      </section>
    </>
  )
}

export default VehicleDashboard
