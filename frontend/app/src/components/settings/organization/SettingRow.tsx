import { useId } from 'react'
import { format } from 'date-fns'
import type { Locale } from 'date-fns'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import {
  Badge,
  Button,
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  Label,
  cn,
} from '@green-ecolution/ui'
import { SettingOriginDto, type SettingChangeRef } from '@/api/backendApi'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { originLabel } from './settingOrigin'

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

export interface SettingRowProps {
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

export default SettingRow
