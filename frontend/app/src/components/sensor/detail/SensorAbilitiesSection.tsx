import { useQuery } from '@tanstack/react-query'
import { Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import { Activity } from 'lucide-react'
import type { SensorModelAbilityResponse } from '@green-ecolution/backend-client'
import { sensorModelIdQuery } from '@/api/queries'
import { getAbilityMeta, getUnitSymbol } from './abilityMapping'
import type { Sensor } from '@/api/backendApi'

interface SensorAbilitiesSectionProps {
  sensor: Sensor
}

interface GroupedAbility {
  ability: string
  unit: string
  depths: number[]
}

const groupAbilities = (abilities: SensorModelAbilityResponse[]): GroupedAbility[] => {
  const groups = new Map<string, GroupedAbility>()
  for (const a of abilities) {
    const existing = groups.get(a.ability)
    if (existing) {
      existing.depths.push(a.depthCm)
    } else {
      groups.set(a.ability, { ability: a.ability, unit: a.unit, depths: [a.depthCm] })
    }
  }
  return Array.from(groups.values()).map((g) => ({
    ...g,
    depths: g.depths.sort((a, b) => a - b),
  }))
}

const SensorAbilitiesSection = ({ sensor }: SensorAbilitiesSectionProps) => {
  const { data: model, isLoading, isError } = useQuery(sensorModelIdQuery(sensor.model.id))

  if (isError) return null

  const abilities = model?.abilities ?? []
  if (!isLoading && abilities.length === 0) return null

  const grouped = groupAbilities(abilities)

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-start gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-light-100 text-green-dark shrink-0">
            <Activity className="size-5" />
          </div>
          <div className="flex flex-col">
            <CardTitle>Was dieser Sensor misst</CardTitle>
            {model?.description && (
              <span className="text-sm text-muted-foreground mt-1.5">{model.description}</span>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <div className="grid gap-4 sm:grid-cols-2">
            {[0, 1].map((i) => (
              <div key={i} className="h-40 rounded-2xl bg-dark-50 animate-pulse" />
            ))}
          </div>
        ) : (
          <ul className="grid gap-4 sm:grid-cols-2">
            {grouped.map((g) => {
              const meta = getAbilityMeta(g.ability)
              const Icon = meta.icon
              return (
                <li
                  key={g.ability}
                  className="group relative overflow-hidden rounded-2xl border border-dark-100 bg-white p-5 transition-shadow hover:shadow-cards"
                >
                  <div
                    aria-hidden
                    className="absolute -right-8 -top-8 size-28 rounded-full bg-green-light-50/70 transition-transform group-hover:scale-110"
                  />
                  <div className="relative flex flex-col gap-3">
                    <div className="grid place-items-center size-12 rounded-xl bg-green-dark text-white shadow-sm">
                      <Icon className="size-6" strokeWidth={1.75} />
                    </div>
                    <h3 className="font-lato font-bold text-lg tracking-tight">{meta.label}</h3>
                    <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-sm">
                      <dt className="text-muted-foreground">Einheit</dt>
                      <dd className="font-mono font-semibold">{getUnitSymbol(g.unit)}</dd>
                      <dt className="text-muted-foreground">
                        {g.depths.length === 1 ? 'Tiefe' : 'Tiefen'}
                      </dt>
                      <dd className="font-mono font-semibold">
                        {g.depths.map((d) => `${d} cm`).join(' · ')}
                      </dd>
                    </dl>
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorAbilitiesSection
