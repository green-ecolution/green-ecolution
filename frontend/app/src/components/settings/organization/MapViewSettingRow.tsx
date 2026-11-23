import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Button, Input, Label, Switch } from '@green-ecolution/ui'
import type { MapViewDto } from '@/api/backendApi'
import MapPreview from '@/components/map-gl/MapPreview'
import { MAP_ZOOM_SCALE_MAX, MAP_ZOOM_SCALE_MIN } from '@/lib/mapConfig'
import MapViewPicker from './MapViewPicker'
import { sameMapView } from './useOrganizationSettingsDraft'

interface MapViewSettingRowProps {
  label: string
  description: string
  value: MapViewDto
  usual: MapViewDto
  locked: boolean
  error: string | null
  onChange: (view: MapViewDto) => void
  onRestrictedChange: (restricted: boolean) => void
  onZoomChange: (which: 'minZoom' | 'maxZoom', level: number) => void
  onUseUsual: () => void
}

const formatCentre = (view: MapViewDto): string =>
  `${view.center[0].toFixed(4)}, ${view.center[1].toFixed(4)}`

interface ZoomFieldProps {
  id: string
  label: string
  value: number | null | undefined
  editable: boolean
  onChange: (level: number) => void
}

/** One end of the zoom range: a field while it may be changed,
 *  a plain figure otherwise. */
const ZoomField = ({ id, label, value, editable, onChange }: ZoomFieldProps) => (
  <div className="flex flex-col gap-1">
    {editable ? (
      <Label htmlFor={id} className="text-sm font-normal text-dark-600">
        {label}
      </Label>
    ) : (
      <p className="text-sm text-dark-600">{label}</p>
    )}
    {editable ? (
      <Input
        id={id}
        className="w-20"
        type="number"
        inputMode="numeric"
        min={MAP_ZOOM_SCALE_MIN}
        max={MAP_ZOOM_SCALE_MAX}
        step={1}
        value={value ?? ''}
        onChange={(event) => {
          const level = Number(event.target.value)
          if (Number.isFinite(level)) onChange(level)
        }}
      />
    ) : (
      <p className="flex h-10 w-20 items-center rounded-lg bg-dark-50 px-3 font-lato text-base font-semibold text-dark">
        {value ?? '—'}
      </p>
    )}
  </div>
)

/** The map viewport: a map rather than four coordinate fields nobody can picture. */
const MapViewSettingRow = ({
  label,
  description,
  value,
  usual,
  locked,
  error,
  onChange,
  onRestrictedChange,
  onZoomChange,
  onUseUsual,
}: MapViewSettingRowProps) => {
  const { t } = useTranslation('settings')
  const fieldId = useId()
  const switchId = useId()
  const editable = !locked
  const restricted = value.bbox != null

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-2">
        <p className="text-sm font-medium text-dark">{label}</p>
        {/* Reads as a statement in both states, so the row says what is the case
            rather than what the switch would do. */}
        <div className="flex items-center gap-3">
          <Label htmlFor={switchId} className="text-sm font-normal text-dark-600">
            {t('organization.settings.mapView.unrestrictedLabel')}
          </Label>
          <Switch
            id={switchId}
            checked={!restricted}
            disabled={!editable}
            onCheckedChange={(checked) => onRestrictedChange(!checked)}
          />
        </div>
      </div>

      <p id={`${fieldId}-hint`} className="max-w-[62ch] text-sm text-dark-600">
        {description}
      </p>

      <div className="mt-1 flex flex-col gap-3">
        {editable ? (
          // Keyed on whether a limit applies: switching that changes what the
          // map is for, and it should be re-framed rather than nudged.
          <MapViewPicker
            key={restricted ? 'bounded' : 'unbounded'}
            value={value}
            onApply={onChange}
          />
        ) : (
          <MapPreview
            bounds={
              value.bbox
                ? [
                    [value.bbox[1], value.bbox[0]],
                    [value.bbox[3], value.bbox[2]],
                  ]
                : undefined
            }
            center={value.bbox ? undefined : [value.center[1], value.center[0]]}
            zoom={12}
            className="h-64"
            ariaLabel={t('organization.settings.mapView.previewAriaLabel')}
          />
        )}

        {restricted && (
          <div className="flex flex-wrap items-end gap-x-4 gap-y-2">
            <ZoomField
              id={`${fieldId}-min-zoom`}
              label={t('organization.settings.mapView.minZoomLabel')}
              value={value.minZoom}
              editable={editable}
              onChange={(level) => onZoomChange('minZoom', level)}
            />
            <ZoomField
              id={`${fieldId}-max-zoom`}
              label={t('organization.settings.mapView.maxZoomLabel')}
              value={value.maxZoom}
              editable={editable}
              onChange={(level) => onZoomChange('maxZoom', level)}
            />
            <p className="max-w-[40ch] text-sm text-dark-600">
              {t('organization.settings.mapView.zoomHint')}
            </p>
          </div>
        )}

        <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
          <p className="flex h-10 items-center rounded-lg bg-dark-50 px-3 font-lato text-base font-semibold text-dark">
            {t('organization.settings.mapView.centre', { value: formatCentre(value) })}
          </p>
          {editable && !sameMapView(value, usual) && (
            <Button type="button" variant="link" onClick={onUseUsual}>
              {t('organization.settings.mapView.useUsual')}
            </Button>
          )}
        </div>
      </div>

      {error && (
        <p
          id={`${fieldId}-error`}
          role="alert"
          aria-live="assertive"
          className="text-sm text-destructive"
        >
          {error}
        </p>
      )}
    </div>
  )
}

export default MapViewSettingRow
