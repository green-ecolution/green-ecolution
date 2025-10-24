import BackLink from '../general/links/BackLink'
import Pill from '../general/Pill'
import ButtonLink from '../general/links/ButtonLink'
import { Pencil } from 'lucide-react'
import { getVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import GeneralLink from '../general/links/GeneralLink'
import { Vehicle, VehicleStatus } from '@green-ecolution/backend-client'
import { getVehicleType } from '@/hooks/details/useDetailsForVehicleType'
import DetailedList from '../general/DetailedList'

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
      <BackLink link={{ to: '/vehicles' }} label="Alle Fahrzeuge" />
      <article className="space-y-6 2xl:space-y-0 2xl:flex 2xl:items-center 2xl:space-x-10">
        <div className="2xl:w-4/5">
          <h1 className="font-lato font-bold text-3xl mb-4 flex flex-wrap items-center gap-4 lg:text-4xl xl:text-5xl">
            Fahrzeug: {vehicle.numberPlate}
            <Pill
              label={statusDetails?.label ?? 'Keine Angabe'}
              theme={statusDetails?.color ?? 'dark-400'}
            />
          </h1>
          {vehicle.description && <p className="mb-4">{vehicle.description}</p>}
        </div>
        <ButtonLink
          icon={Pencil}
          iconClassName="stroke-1"
          label="Fahrzeug bearbeiten"
          color="grey"
          link={{
            to: `/vehicles/$vehicleId/edit`,
            params: { vehicleId: String(vehicle.id) },
          }}
        />
      </article>

      {vehicle.status == VehicleStatus.VehicleStatusActive && (
        <div className="h-full shadow-cards space-y-3 rounded-xl border border-green-light bg-green-light-50 p-6 mt-6">
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
