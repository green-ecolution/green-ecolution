import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle, cn } from '@green-ecolution/ui'
import type { SensorModelAbilityResponse } from '@green-ecolution/backend-client'
import { sensorQueries } from '@/api/queries'
import { depthColor } from '@/components/general/charts/soilMoistureChart'
import { useAbilityMeta, getUnitSymbol } from './abilityMapping'
import { compactCardContent, compactCardHeader, compactCardTitle } from './cardDensity'
import type { Sensor } from '@/api/backendApi'

interface SensorMeasurementProfileProps {
  sensor: Sensor
}

interface DepthLayer {
  depthCm: number
  abilities: SensorModelAbilityResponse[]
  color: string | null
}

const toDepthLayers = (abilities: SensorModelAbilityResponse[]): DepthLayer[] => {
  const byDepth = new Map<number, SensorModelAbilityResponse[]>()
  for (const ability of abilities) {
    byDepth.set(ability.depthCm, [...(byDepth.get(ability.depthCm) ?? []), ability])
  }
  // Only depths that also appear as a curve in the soil moisture chart get its colour,
  // indexed the same way, so dot and curve can be matched at a glance.
  const chartDepths = abilities
    .filter((a) => a.ability === 'soil_moisture')
    .map((a) => a.depthCm)
    .sort((a, b) => a - b)
  return [...byDepth.entries()]
    .sort(([a], [b]) => a - b)
    .map(([depthCm, layerAbilities]) => {
      const chartIndex = chartDepths.indexOf(depthCm)
      return {
        depthCm,
        abilities: layerAbilities,
        color: chartIndex === -1 ? null : depthColor(depthCm, chartIndex),
      }
    })
}

const SensorMeasurementProfile = ({ sensor }: SensorMeasurementProfileProps) => {
  const { t } = useTranslation('sensor')
  const getAbilityMeta = useAbilityMeta()
  const { data: model, isLoading, isError } = useQuery(sensorQueries.model(sensor.model.id))

  if (isError) return null
  // Battery voltage is device health, already shown in the KPI row, not a soil reading.
  const abilities = (model?.abilities ?? []).filter((a) => a.ability !== 'battery')
  if (!isLoading && abilities.length === 0) return null

  const layers = toDepthLayers(abilities)

  return (
    <Card variant="outlined" className="h-full">
      <CardHeader className={compactCardHeader}>
        <CardTitle className={compactCardTitle}>{t('profile.title')}</CardTitle>
      </CardHeader>
      <CardContent className={compactCardContent}>
        {isLoading ? (
          <div className="h-28 rounded-xl bg-dark-50 animate-pulse" />
        ) : (
          <ol className="relative flex flex-col gap-3 before:absolute before:top-1 before:bottom-2 before:left-[9px] before:w-0.5 before:bg-dark-200">
            <li className="flex items-center gap-3">
              <span className="relative h-0.5 w-5 rounded-full bg-dark-400" aria-hidden />
              <span className="text-xs text-muted-foreground">{t('profile.surfaceLabel')}</span>
            </li>
            {layers.map((layer) => (
              <li key={layer.depthCm} className="flex gap-3">
                <span className="relative flex w-5 shrink-0 justify-center pt-1">
                  <span
                    aria-hidden
                    className={cn(
                      'size-3 rounded-full ring-4 ring-card',
                      !layer.color && 'bg-dark-400',
                    )}
                    style={layer.color ? { backgroundColor: layer.color } : undefined}
                  />
                </span>
                <div className="flex min-w-0 flex-1 flex-wrap items-baseline gap-x-3 gap-y-0.5">
                  <span className="w-14 shrink-0 font-semibold tabular-nums">
                    {t('profile.depthLabel', { depth: layer.depthCm })}
                  </span>
                  <ul className="flex min-w-0 flex-wrap gap-x-3 gap-y-0.5 text-sm">
                    {layer.abilities.map((a) => {
                      const meta = getAbilityMeta(a.ability)
                      const Icon = meta.icon
                      return (
                        <li key={a.ability} className="inline-flex items-center gap-1.5">
                          <Icon className="size-4 text-green-dark" aria-hidden />
                          <span>{meta.label}</span>
                          <span className="text-muted-foreground">{getUnitSymbol(a.unit)}</span>
                        </li>
                      )
                    })}
                  </ul>
                </div>
              </li>
            ))}
          </ol>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorMeasurementProfile
