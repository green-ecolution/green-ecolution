import { useId } from 'react'
import { format } from 'date-fns'
import type { Locale } from 'date-fns'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import { Badge, Button, Input, Label, Switch, cn } from '@green-ecolution/ui'
import { SettingOriginDto, type MapViewDto, type SettingChangeRef } from '@/api/backendApi'
import MapPreview from '@/components/map-gl/MapPreview'
import { MAP_ZOOM_SCALE_MAX, MAP_ZOOM_SCALE_MIN } from '@/lib/mapConfig'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import MapViewPicker from './MapViewPicker'

interface MapViewSettingRowProps {
  label: string
  description: string
  /** The viewport in force, shown while this organization does not own it. */
  effective: MapViewDto
  /** The viewport being edited; equals `effective` until the person moves it. */
  draft: MapViewDto
  origin: SettingOriginDto
  sourceName: string | null
  /** Own value while a lock above keeps it from taking effect. */
  dormant: MapViewDto | null
  lastChange: SettingChangeRef | null | undefined
  own: boolean
  locked: boolean
  error: string | null
  onOwn: (own: boolean) => void
  onChange: (view: MapViewDto) => void
  onRestrictedChange: (restricted: boolean) => void
  onZoomChange: (which: 'minZoom' | 'maxZoom', level: number) => void
}

const originLabel = (
  origin: SettingOriginDto,
  sourceName: string | null,
  t: TFunction<'settings'>,
): string => {
  switch (origin) {
    case SettingOriginDto.Own:
      return t('organization.settings.origin.own')
    case SettingOriginDto.Inherited:
      return sourceName
        ? t('organization.settings.origin.inherited', { name: sourceName })
        : t('organization.settings.origin.inheritedUnknown')
    case SettingOriginDto.Default:
      return t('organization.settings.origin.default')
  }
}

const lastChangeLine = (
  change: SettingChangeRef | null | undefined,
  locale: Locale,
  t: TFunction<'settings'>,
): string | null => {
  if (!change) return null
  const changedAt = new Date(change.changedAt)
  if (Number.isNaN(changedAt.getTime())) return null
  const date = format(changedAt, 'dd.MM.yyyy', { locale })
  return change.changedByName
    ? t('organization.settings.lastChangeBy', { date, name: change.changedByName })
    : t('organization.settings.lastChange', { date })
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

/** One end of the zoom range: a field while this organization owns the value,
 *  a plain figure everywhere else, like the numeric rows above. */
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

/**
 * The map viewport, in the same shape as the numeric rows next to it: origin
 * rail, badge, "set my own" / "inherit again". Only the value itself is a map
 * rather than a number, because four coordinate fields are not something a
 * person can picture.
 */
const MapViewSettingRow = ({
  label,
  description,
  effective,
  draft,
  origin,
  sourceName,
  dormant,
  lastChange,
  own,
  locked,
  error,
  onOwn,
  onChange,
  onRestrictedChange,
  onZoomChange,
}: MapViewSettingRowProps) => {
  const { t } = useTranslation('settings')
  const dateLocale = useDateLocale()
  const fieldId = useId()
  const switchId = useId()
  const history = lastChangeLine(lastChange, dateLocale, t)
  // Same rule as the numeric rows: a value is a field only while this
  // organization owns it and may change it; elsewhere it is a fact to read.
  const editable = own && !locked
  const shown = editable ? draft : effective
  const restricted = shown.bbox != null

  return (
    <div
      className={cn(
        'flex flex-col gap-2 border-l-2 pl-4',
        origin === SettingOriginDto.Own ? 'border-green-dark' : 'border-dark-200',
      )}
    >
      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-2">
        <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          <p className="text-sm font-medium text-dark">{label}</p>
          <Badge variant={origin === SettingOriginDto.Own ? 'outline-green-dark' : 'muted'}>
            {originLabel(origin, sourceName, t)}
          </Badge>
        </div>
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
            value={draft}
            onApply={onChange}
          />
        ) : (
          <MapPreview
            bounds={
              shown.bbox
                ? [
                    [shown.bbox[1], shown.bbox[0]],
                    [shown.bbox[3], shown.bbox[2]],
                  ]
                : undefined
            }
            center={shown.bbox ? undefined : [shown.center[1], shown.center[0]]}
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
              value={shown.minZoom}
              editable={editable}
              onChange={(level) => onZoomChange('minZoom', level)}
            />
            <ZoomField
              id={`${fieldId}-max-zoom`}
              label={t('organization.settings.mapView.maxZoomLabel')}
              value={shown.maxZoom}
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
            {t('organization.settings.mapView.centre', { value: formatCentre(shown) })}
          </p>
          {!locked && (
            <Button
              type="button"
              variant="link"
              size="sm"
              className="h-auto px-0 text-sm"
              aria-label={
                own
                  ? t('organization.settings.inheritAgainFor', { field: label })
                  : t('organization.settings.setOwnValueFor', { field: label })
              }
              onClick={() => onOwn(!own)}
            >
              {own
                ? t('organization.settings.inheritAgain')
                : t('organization.settings.setOwnValue')}
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
      {history && <p className="text-xs text-dark-600">{history}</p>}
      {dormant && (
        <p className="text-xs text-dark-600">
          {t('organization.settings.dormantOwnValue', { value: formatCentre(dormant) })}
        </p>
      )}
    </div>
  )
}

export default MapViewSettingRow
