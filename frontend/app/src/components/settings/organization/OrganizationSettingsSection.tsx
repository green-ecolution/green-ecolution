import { useId } from 'react'
import { format } from 'date-fns'
import type { Locale } from 'date-fns'
import { Lock, LockOpen } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import {
  justWateredTtlMaxHours,
  justWateredTtlMinHours,
  waterDemandMax,
  waterDemandMin,
} from '@green-ecolution/domain-wasm'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Badge,
  Button,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  Label,
  Separator,
  Switch,
  cn,
} from '@green-ecolution/ui'
import {
  SettingOriginDto,
  type MapViewDto,
  type OrganizationSettingsResponse,
  type SettingChangeRef,
} from '@/api/backendApi'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { CARD, CARD_TITLE } from './OrganizationDetail'
import MapViewSettingRow from './MapViewSettingRow'
import { hoursOf, type NumericField, type SettingsDraft } from './useOrganizationSettingsDraft'

export interface SettingsFieldErrors {
  waterDemand: string | null
  justWateredTtlHours: string | null
  mapView: string | null
}

export type SettingsSourceNames = Record<NumericField | 'mapView', string | null>

interface OrganizationSettingsSectionProps {
  settings: OrganizationSettingsResponse
  draft: SettingsDraft
  errors: SettingsFieldErrors
  canUpdate: boolean
  /** Name of the organization that froze this subtree, if any. */
  enforcedByName: string | null
  sourceNames: SettingsSourceNames
  onOwnChange: (field: NumericField, own: boolean) => void
  onTextChange: (field: NumericField, text: string) => void
  onMapViewOwnChange: (own: boolean) => void
  onMapViewChange: (view: MapViewDto) => void
  onMapViewRestrictedChange: (restricted: boolean) => void
  onMapViewZoomChange: (which: 'minZoom' | 'maxZoom', level: number) => void
  onDescendantsMayOverrideChange: (value: boolean) => void
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

interface SettingRowProps {
  label: string
  description: string
  /** Unit of the input, e.g. "Liter"; the read state carries its own wording. */
  unit: string
  /** The value in effect, already worded with its unit, for the read state. */
  effective: string
  origin: SettingOriginDto
  sourceName: string | null
  /** Own value while a lock above keeps it from taking effect. */
  dormantValue: string | null
  lastChange: SettingChangeRef | null | undefined
  text: string
  own: boolean
  error: string | null
  locked: boolean
  min: number
  max: number
  step: number
  onOwn: (own: boolean) => void
  onText: (text: string) => void
}

const SettingRow = ({
  label,
  description,
  unit,
  effective,
  origin,
  sourceName,
  dormantValue,
  lastChange,
  text,
  own,
  error,
  locked,
  min,
  max,
  step,
  onOwn,
  onText,
}: SettingRowProps) => {
  const { t } = useTranslation('settings')
  const dateLocale = useDateLocale()
  const fieldId = useId()
  const history = lastChangeLine(lastChange, dateLocale, t)
  // A value is a field only while this organization owns it and may change it.
  // Everywhere else it is a fact to read, not a greyed-out box to puzzle over.
  const editable = own && !locked
  const notes = [error && `${fieldId}-error`, `${fieldId}-hint`, `${fieldId}-unit`].filter(Boolean)

  return (
    <div
      className={cn(
        'flex flex-col gap-2 border-l-2 pl-4',
        // The rail carries the origin down the column, so ownership reads at a
        // glance without parsing either badge.
        origin === SettingOriginDto.Own ? 'border-green-dark' : 'border-dark-200',
      )}
    >
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        {editable ? (
          <Label htmlFor={fieldId}>{label}</Label>
        ) : (
          <p className="text-sm font-medium text-dark">{label}</p>
        )}
        {/* Colour and wording both come from the stored origin: an unsaved
            takeover has not moved the value yet, and the action below already
            says what the person is about to do. */}
        <Badge variant={origin === SettingOriginDto.Own ? 'outline-green-dark' : 'muted'}>
          {originLabel(origin, sourceName, t)}
        </Badge>
      </div>

      <p id={`${fieldId}-hint`} className="max-w-[62ch] text-sm text-dark-600">
        {description}
      </p>

      <div className="mt-1 flex flex-wrap items-center gap-x-4 gap-y-2">
        {editable ? (
          <InputGroup className="w-fit">
            <InputGroupInput
              id={fieldId}
              className="w-24 flex-none"
              type="number"
              inputMode="decimal"
              min={min}
              max={max}
              step={step}
              value={text}
              aria-invalid={!!error}
              aria-describedby={notes.join(' ')}
              onChange={(event) => onText(event.target.value)}
            />
            <InputGroupAddon align="inline-end">
              <InputGroupText id={`${fieldId}-unit`}>{unit}</InputGroupText>
            </InputGroupAddon>
          </InputGroup>
        ) : (
          <p className="flex h-10 items-center rounded-lg bg-dark-50 px-3 font-lato text-base font-semibold text-dark">
            {effective}
          </p>
        )}
        {!locked && (
          <Button
            type="button"
            variant="link"
            size="sm"
            className="h-auto px-0 text-sm"
            // Both fields can inherit at once, and then two identically named
            // actions sit side by side with nothing tying either to its row.
            aria-label={
              own
                ? t('organization.settings.inheritAgainFor', { field: label })
                : t('organization.settings.setOwnValueFor', { field: label })
            }
            onClick={() => onOwn(!own)}
          >
            {own ? t('organization.settings.inheritAgain') : t('organization.settings.setOwnValue')}
          </Button>
        )}
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
      {dormantValue && (
        <p className="text-xs text-dark-600">
          {t('organization.settings.dormantOwnValue', { value: dormantValue })}
        </p>
      )}
    </div>
  )
}

const OrganizationSettingsSection = ({
  settings,
  draft,
  errors,
  canUpdate,
  enforcedByName,
  sourceNames,
  onOwnChange,
  onTextChange,
  onMapViewOwnChange,
  onMapViewChange,
  onMapViewRestrictedChange,
  onMapViewZoomChange,
  onDescendantsMayOverrideChange,
}: OrganizationSettingsSectionProps) => {
  const { t } = useTranslation('settings')
  const switchId = useId()
  // Read off the response, not off the name: an organization above the visible
  // list locks just the same, it just cannot be named.
  const enforced = settings.enforcedBy != null
  // The lock covers the switch itself — the backend refuses the whole write,
  // so a sub-unit cannot unlock itself.
  const locked = enforced || !canUpdate

  const waterDemandOwnValue = settings.waterDemand.ownValue
  const ttlOwnValue = settings.justWateredTtlSecs.ownValue
  const liters = (value: number) => t('organization.settings.liters', { count: value })
  const hours = (seconds: number) => t('organization.settings.hours', { count: hoursOf(seconds) })
  const descendantsMayOverride = draft.descendantsMayOverride

  return (
    <section className={CARD}>
      <h3 className={CARD_TITLE}>{t('organization.settings.title')}</h3>
      <p className="mt-1 max-w-[70ch] text-sm text-dark-600">{t('organization.settings.hint')}</p>

      {enforced ? (
        <Alert variant="info" size="default" className="mt-4 flex w-full gap-4" role="alert">
          <AlertIcon variant="info" icon={Lock} />
          <AlertContent>
            <AlertDescription>
              {enforcedByName
                ? t('organization.settings.enforcedNotice', { name: enforcedByName })
                : t('organization.settings.enforcedNoticeUnknown')}
            </AlertDescription>
          </AlertContent>
        </Alert>
      ) : (
        // Without the permission the take-over actions are gone; say why rather
        // than leave an unexplained gap where they were.
        !canUpdate && (
          <p className="mt-2 text-sm text-dark-600">{t('organization.settings.readOnlyNotice')}</p>
        )
      )}

      <div className="mt-5 grid gap-6 @min-[48rem]:grid-cols-2">
        <SettingRow
          label={t('organization.settings.waterDemandLabel')}
          description={t('organization.settings.waterDemandHint')}
          unit={t('organization.settings.waterDemandUnit')}
          effective={liters(settings.waterDemand.value)}
          origin={settings.waterDemand.origin}
          sourceName={sourceNames.waterDemand}
          dormantValue={
            enforced && waterDemandOwnValue != null ? liters(waterDemandOwnValue) : null
          }
          lastChange={settings.waterDemand.lastChange}
          text={draft.waterDemand.text}
          own={draft.waterDemand.own}
          error={errors.waterDemand}
          locked={locked}
          min={waterDemandMin()}
          max={waterDemandMax()}
          step={1}
          onOwn={(own) => onOwnChange('waterDemand', own)}
          onText={(value) => onTextChange('waterDemand', value)}
        />
        <SettingRow
          label={t('organization.settings.justWateredTtlLabel')}
          description={t('organization.settings.justWateredTtlHint')}
          unit={t('organization.settings.justWateredTtlUnit')}
          effective={hours(settings.justWateredTtlSecs.value)}
          origin={settings.justWateredTtlSecs.origin}
          sourceName={sourceNames.justWateredTtlHours}
          dormantValue={enforced && ttlOwnValue != null ? hours(ttlOwnValue) : null}
          lastChange={settings.justWateredTtlSecs.lastChange}
          text={draft.justWateredTtlHours.text}
          own={draft.justWateredTtlHours.own}
          error={errors.justWateredTtlHours}
          locked={locked}
          min={justWateredTtlMinHours()}
          max={justWateredTtlMaxHours()}
          step={1}
          onOwn={(own) => onOwnChange('justWateredTtlHours', own)}
          onText={(value) => onTextChange('justWateredTtlHours', value)}
        />
      </div>

      {/* Its own row rather than a third cell: a map needs the full width to be
          worth looking at. */}
      <div className="mt-6">
        <MapViewSettingRow
          label={t('organization.settings.mapView.label')}
          description={t('organization.settings.mapView.hint')}
          effective={settings.mapView.value}
          draft={draft.mapView.value}
          origin={settings.mapView.origin}
          sourceName={sourceNames.mapView}
          dormant={enforced ? (settings.mapView.ownValue ?? null) : null}
          lastChange={settings.mapView.lastChange}
          own={draft.mapView.own}
          locked={locked}
          error={errors.mapView}
          onOwn={onMapViewOwnChange}
          onChange={onMapViewChange}
          onRestrictedChange={onMapViewRestrictedChange}
          onZoomChange={onMapViewZoomChange}
        />
      </div>

      <Separator className="my-5" />

      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-3">
        <div className="flex min-w-0 flex-1 gap-3">
          {descendantsMayOverride ? (
            <LockOpen className="mt-0.5 size-4 shrink-0 text-dark-500" aria-hidden />
          ) : (
            <Lock className="mt-0.5 size-4 shrink-0 text-dark" aria-hidden />
          )}
          <div className="min-w-0">
            <Label htmlFor={switchId}>{t('organization.settings.descendantsLabel')}</Label>
            {/* The consequence of switching it off reaches the whole subtree, so
                spell out the state the person is in rather than both at once. */}
            <p className="mt-1 max-w-[62ch] text-sm text-dark-600">
              {descendantsMayOverride
                ? t('organization.settings.descendantsHintOn')
                : t('organization.settings.descendantsHintOff')}
            </p>
          </div>
        </div>
        <Switch
          id={switchId}
          checked={descendantsMayOverride}
          disabled={locked}
          onCheckedChange={onDescendantsMayOverrideChange}
        />
      </div>
    </section>
  )
}

export default OrganizationSettingsSection
