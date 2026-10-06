import { useTranslation } from 'react-i18next'
import { Badge } from '@green-ecolution/ui'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { getSensorImage } from './sensorImages'
import SensorActionsMenu from './SensorActionsMenu'
import type { Sensor } from '@/api/backendApi'

interface SensorHeroProps {
  sensor: Sensor
}

const SensorHero = ({ sensor }: SensorHeroProps) => {
  const { t } = useTranslation('sensor')
  const image = getSensorImage(sensor.model.name)
  const getSensorStatusDetails = useSensorStatusDetails()
  const status = getSensorStatusDetails(sensor.status)
  const sensorTypeLabel = sensor.sensorType === 'lorawan' ? 'LoRaWAN' : sensor.sensorType

  return (
    <header className="flex flex-wrap items-start gap-x-5 gap-y-4">
      <div className="aspect-[3/2] w-32 shrink-0 overflow-hidden rounded-xl border border-dark-100 bg-white md:w-48 xl:w-56">
        <img
          src={image}
          alt={t('hero.modelImageAlt', { model: sensor.model.name })}
          className="size-full object-cover"
          loading="lazy"
        />
      </div>

      <div className="min-w-0 flex-1 basis-40 self-center">
        <h1 className="font-lato text-2xl font-bold leading-tight tracking-tight break-all md:text-3xl xl:text-4xl">
          {sensor.id}
        </h1>
        <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1.5 text-sm">
          <Badge variant={status.color} className="gap-1.5">
            <span className="size-1.5 rounded-full bg-current" aria-hidden />
            {status.label}
          </Badge>
          <span className="font-semibold">{sensor.model.name}</span>
          <span className="text-muted-foreground">{sensorTypeLabel}</span>
          {sensor.provider && (
            <span className="text-muted-foreground">
              {t('hero.viaLabel')} <span className="text-foreground">{sensor.provider}</span>
            </span>
          )}
        </div>
      </div>

      <div className="w-full sm:w-auto sm:self-start">
        <SensorActionsMenu sensor={sensor} />
      </div>
    </header>
  )
}

export default SensorHero
