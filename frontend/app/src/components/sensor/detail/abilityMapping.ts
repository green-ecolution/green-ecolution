import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
  CloudDrizzle,
  Droplets,
  Gauge,
  HelpCircle,
  Thermometer,
  type LucideIcon,
} from 'lucide-react'

export interface AbilityMeta {
  label: string
  icon: LucideIcon
}

// `t`'s generated overloads only accept the catalog's literal key union; the
// ability plugged into the template isn't statically one of those literals.
type SensorTranslate = (key: string) => string

const ABILITY_ICON: Record<string, LucideIcon> = {
  soil_moisture: Droplets,
  soil_tension: Gauge,
  temperature: Thermometer,
  humidity: CloudDrizzle,
}

const UNIT_SYMBOL: Record<string, string> = {
  percent: '%',
  centibar: 'cbar',
  celsius: '°C',
  ohm: 'Ω',
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useAbilityMeta = (): ((ability: string) => AbilityMeta) => {
  const { t } = useTranslation('sensor')
  const translate = t as SensorTranslate
  return useCallback(
    (ability: string): AbilityMeta => {
      const icon = ABILITY_ICON[ability] ?? HelpCircle
      return ability in ABILITY_ICON
        ? { label: translate(`ability.${ability}`), icon }
        : { label: ability, icon }
    },
    [translate],
  )
}

export const getUnitSymbol = (unit: string): string => UNIT_SYMBOL[unit] ?? unit
