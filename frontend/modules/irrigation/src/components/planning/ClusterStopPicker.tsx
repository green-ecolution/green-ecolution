import { useNavigate } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useMapCenter, useMapZoom } from '@/components/map-gl/useMapView'
import type { StopPickerProps } from '@/modules/types'
import SelectEntities from '../general/form/types/SelectEntities'
import { useShownStops } from '../../hooks/useShownStops'

const ClusterStopPicker = ({ value, onChange, error, form }: StopPickerProps) => {
  const { t } = useTranslation('irrigation')
  const navigate = useNavigate()
  const [lat, lng] = useMapCenter()
  const zoom = useMapZoom()
  const { shown, keepUnshown } = useShownStops(value)

  const pickOnMap = () => {
    form.saveDraft()
    navigate({
      to: '/map/watering-plan/select/cluster',
      search: {
        lat,
        lng,
        zoom,
        wateringPlanId: form.planId,
        transporterId: form.transporterId,
        trailerId: form.trailerId,
        clusterIds: value,
        formType: form.type,
      },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <div>
      <SelectEntities
        onChange={(next) => onChange(keepUnshown(next))}
        entityIds={shown}
        onAdd={pickOnMap}
        type="cluster"
        label={t('planning.stopPicker.label')}
      />
      {error && (
        <p role="alert" aria-live="assertive" className="mt-2 text-sm text-destructive">
          {t('planning.stopPicker.requiredError')}
        </p>
      )}
    </div>
  )
}

export default ClusterStopPicker
