import ges1000 from '@/assets/sensors/ges-1000.png'
import placeholder from '@/assets/sensors/placeholder.svg'

const SENSOR_IMAGES: Record<string, string> = {
  'GES-1000': ges1000,
}

export const getSensorImage = (modelName?: string | null): string =>
  (modelName ? SENSOR_IMAGES[modelName] : undefined) ?? placeholder
