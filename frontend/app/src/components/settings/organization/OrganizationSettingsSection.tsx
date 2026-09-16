import { useId } from 'react'
import { format } from 'date-fns'
import type { Locale } from 'date-fns'
import { Lock } from 'lucide-react'
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
  FormField,
  Label,
  Separator,
  Switch,
} from '@green-ecolution/ui'
import {
  SettingOriginDto,
  type OrganizationSettingsResponse,
  type SettingChangeRef,
} from '@/api/backendApi'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { CARD, CARD_TITLE } from './OrganizationDetail'
import { hoursOf, type NumericField, type SettingsDraft } from './useOrganizationSettingsDraft'

export interface SettingsFieldErrors {
  waterDemand: string | null
  justWateredTtlHours: string | null
}

export type SettingsSourceNames = Record<NumericField, string | null>

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
  const history = lastChangeLine(lastChange, dateLocale, t)

  return (
    <div className="flex flex-col gap-2">
      {/* Colour and wording both come from the stored origin: an unsaved
          takeover has not moved the value yet, and the toggle below already
          shows what the person is about to do. */}
      <Badge variant={origin === 'own' ? 'outline-green-dark' : 'muted'} className="self-start">
        {originLabel(origin, sourceName, t)}
      </Badge>
      <FormField
        label={label}
        type="number"
        inputMode="decimal"
        min={min}
        max={max}
        step={step}
        value={text}
        disabled={locked || !own}
        error={error ?? undefined}
        onChange={(event) => onText(event.target.value)}
      />
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={locked}
          // Both fields can inherit at once, and then two identically named
          // buttons sit side by side with nothing tying either to its row.
          aria-label={
            own
              ? t('organization.settings.inheritAgainFor', { field: label })
              : t('organization.settings.setOwnValueFor', { field: label })
          }
          onClick={() => onOwn(!own)}
        >
          {own ? t('organization.settings.inheritAgain') : t('organization.settings.setOwnValue')}
        </Button>
        {history && <p className="text-xs text-dark-600">{history}</p>}
      </div>
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

  return (
    <section className={CARD}>
      <h3 className={CARD_TITLE}>{t('organization.settings.title')}</h3>
      <p className="mt-0.5 text-sm text-dark-600">{t('organization.settings.hint')}</p>

      {enforced && (
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
      )}

      <div className="mt-4 grid gap-6 @min-[48rem]:grid-cols-2">
        <SettingRow
          label={t('organization.settings.waterDemandLabel')}
          origin={settings.waterDemand.origin}
          sourceName={sourceNames.waterDemand}
          dormantValue={
            enforced && waterDemandOwnValue != null
              ? t('organization.settings.liters', { count: waterDemandOwnValue })
              : null
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
          origin={settings.justWateredTtlSecs.origin}
          sourceName={sourceNames.justWateredTtlHours}
          dormantValue={
            enforced && ttlOwnValue != null
              ? t('organization.settings.hours', { count: hoursOf(ttlOwnValue) })
              : null
          }
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

      <Separator className="my-5" />

      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <Label htmlFor={switchId}>{t('organization.settings.descendantsLabel')}</Label>
          <p className="mt-0.5 text-sm text-dark-600">
            {t('organization.settings.descendantsHint')}
          </p>
        </div>
        <Switch
          id={switchId}
          checked={draft.descendantsMayOverride}
          disabled={locked}
          onCheckedChange={onDescendantsMayOverrideChange}
        />
      </div>
    </section>
  )
}

export default OrganizationSettingsSection
