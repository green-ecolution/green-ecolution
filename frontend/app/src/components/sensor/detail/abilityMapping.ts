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

const ABILITY_META: Record<string, AbilityMeta> = {
  soil_moisture: { label: 'Bodenfeuchtigkeit', icon: Droplets },
  soil_tension: { label: 'Bodenspannung', icon: Gauge },
  temperature: { label: 'Temperatur', icon: Thermometer },
  humidity: { label: 'Luftfeuchtigkeit', icon: CloudDrizzle },
}

const UNIT_SYMBOL: Record<string, string> = {
  percent: '%',
  centibar: 'cbar',
  celsius: '°C',
  ohm: 'Ω',
}

export const getAbilityMeta = (ability: string): AbilityMeta =>
  ABILITY_META[ability] ?? { label: ability, icon: HelpCircle }

export const getUnitSymbol = (unit: string): string => UNIT_SYMBOL[unit] ?? unit
