import { Badge } from '@green-ecolution/ui'
import { getSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { getSensorImage } from './sensorImages'
import SensorActionsMenu from './SensorActionsMenu'
import type { Sensor } from '@/api/backendApi'

interface SensorHeroProps {
  sensor: Sensor
}

const SensorHero = ({ sensor }: SensorHeroProps) => {
  const image = getSensorImage(sensor.model.name)
  const status = getSensorStatusDetails(sensor.status)
  const sensorTypeLabel = sensor.sensorType === 'lorawan' ? 'LoRaWAN' : sensor.sensorType

  return (
    <header className="flex flex-col gap-8 md:flex-row md:items-center md:gap-10">
      <div className="relative shrink-0 self-start">
        <div
          aria-hidden
          className="absolute -inset-4 -z-10 rounded-[2rem] bg-gradient-to-br from-green-light-100/70 via-white to-green-dark-50/80"
        />
        <div className="size-44 md:size-56 rounded-3xl border border-dark-100 bg-white shadow-cards p-5 grid place-items-center">
          <img
            src={image}
            alt={`Modellabbildung: ${sensor.model.name}`}
            className="size-full object-cover"
            loading="lazy"
          />
        </div>
      </div>

      <div className="flex-1 min-w-0">
        <p className="font-lato text-xs font-bold uppercase tracking-[0.22em] text-green-dark mb-3">
          {sensorTypeLabel} Sensor
        </p>
        <h1 className="font-lato font-bold text-3xl lg:text-4xl xl:text-5xl tracking-tight leading-[1.05] break-all">
          {sensor.id}
        </h1>
        <div className="mt-5 flex flex-wrap items-center gap-x-3 gap-y-2">
          <Badge variant={status.color} size="lg" className="gap-2">
            <span className="size-2 rounded-full bg-current" aria-hidden />
            {status.label}
          </Badge>
          <span className="text-dark-300" aria-hidden>
            ·
          </span>
          <span className="font-lato font-semibold text-lg">{sensor.model.name}</span>
          {sensor.provider && (
            <>
              <span className="text-dark-300" aria-hidden>
                ·
              </span>
              <span className="text-sm text-muted-foreground">
                via <span className="font-semibold text-foreground">{sensor.provider}</span>
              </span>
            </>
          )}
        </div>
      </div>

      <div className="md:self-start md:pt-2">
        <SensorActionsMenu sensorId={sensor.id} />
      </div>
    </header>
  )
}

export default SensorHero
