import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
  InputGroupText,
  Label,
} from '@green-ecolution/ui'

export interface SettingValueFieldProps {
  label: string
  description: string
  /** Unit behind the input, e.g. "Liter". */
  unit: string
  /** The inherited value worded with its unit, e.g. "80 Liter". */
  usual: string
  /** Picks the singular or plural sentence around `usual`. */
  usualCount: number
  text: string
  error: string | null
  disabled: boolean
  min: number
  max: number
  step: number
  onText: (text: string) => void
}

const SettingValueField = ({
  label,
  description,
  unit,
  usual,
  usualCount,
  text,
  error,
  disabled,
  min,
  max,
  step,
  onText,
}: SettingValueFieldProps) => {
  const { t } = useTranslation('settings')
  const id = useId()
  const notes = [error && `${id}-error`, `${id}-hint`, `${id}-unit`, `${id}-usual`].filter(Boolean)

  return (
    <div className="flex flex-wrap items-start gap-x-8 gap-y-3 border-t border-dark-100 px-6 py-5">
      <div className="min-w-0 flex-[1_1_18rem]">
        <Label htmlFor={id} className="font-bold">
          {label}
        </Label>
        <p id={`${id}-hint`} className="mt-0.5 max-w-[52ch] text-sm text-dark-600">
          {description}
        </p>
      </div>
      <div className="flex-[0_0_12.5rem]">
        <InputGroup className="w-fit">
          <InputGroupInput
            id={id}
            className="w-24 flex-none"
            type="number"
            inputMode="decimal"
            min={min}
            max={max}
            step={step}
            value={text}
            disabled={disabled}
            aria-invalid={!!error}
            aria-describedby={notes.join(' ')}
            onChange={(event) => onText(event.target.value)}
          />
          <InputGroupAddon align="inline-end">
            <InputGroupText id={`${id}-unit`} className="text-dark">
              {unit}
            </InputGroupText>
          </InputGroupAddon>
        </InputGroup>
        <p id={`${id}-usual`} className="mt-1.5 text-[0.8125rem] text-dark-600">
          {t('organization.areas.usual', { count: usualCount, value: usual })}
        </p>
        {error && (
          <p id={`${id}-error`} role="alert" className="mt-1 text-sm text-destructive">
            {error}
          </p>
        )}
      </div>
    </div>
  )
}

export default SettingValueField
