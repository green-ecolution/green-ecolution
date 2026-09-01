import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { SoilCondition } from '@green-ecolution/backend-client'

// `t`'s generated overloads only accept the catalog's literal key union; the
// enum value plugged into the template isn't statically one of those literals.
type EnumsTranslate = (key: string) => string

type SoilGroupKey = 'sands' | 'silts' | 'loams' | 'clays' | 'pureSands' | 'other'

export interface SoilConditionOption {
  value: SoilCondition
  label: string
  group: string
}

// KA5 fine soil types, grouped by Hauptbodenart.
const SoilConditionEntries: { value: SoilCondition; groupKey: SoilGroupKey }[] = [
  { value: SoilCondition.Ss, groupKey: 'sands' },
  { value: SoilCondition.Su2, groupKey: 'sands' },
  { value: SoilCondition.Su3, groupKey: 'sands' },
  { value: SoilCondition.Su4, groupKey: 'sands' },
  { value: SoilCondition.Sl2, groupKey: 'sands' },
  { value: SoilCondition.Sl3, groupKey: 'sands' },
  { value: SoilCondition.Sl4, groupKey: 'sands' },
  { value: SoilCondition.Slu, groupKey: 'sands' },
  { value: SoilCondition.St2, groupKey: 'sands' },
  { value: SoilCondition.St3, groupKey: 'sands' },
  { value: SoilCondition.Uu, groupKey: 'silts' },
  { value: SoilCondition.Us, groupKey: 'silts' },
  { value: SoilCondition.Uls, groupKey: 'silts' },
  { value: SoilCondition.Ut2, groupKey: 'silts' },
  { value: SoilCondition.Ut3, groupKey: 'silts' },
  { value: SoilCondition.Ut4, groupKey: 'silts' },
  { value: SoilCondition.Ls2, groupKey: 'loams' },
  { value: SoilCondition.Ls3, groupKey: 'loams' },
  { value: SoilCondition.Ls4, groupKey: 'loams' },
  { value: SoilCondition.Lt2, groupKey: 'loams' },
  { value: SoilCondition.Lt3, groupKey: 'loams' },
  { value: SoilCondition.Lts, groupKey: 'loams' },
  { value: SoilCondition.Lu, groupKey: 'loams' },
  { value: SoilCondition.Tt, groupKey: 'clays' },
  { value: SoilCondition.Tu2, groupKey: 'clays' },
  { value: SoilCondition.Tu3, groupKey: 'clays' },
  { value: SoilCondition.Tu4, groupKey: 'clays' },
  { value: SoilCondition.Ts2, groupKey: 'clays' },
  { value: SoilCondition.Ts3, groupKey: 'clays' },
  { value: SoilCondition.Ts4, groupKey: 'clays' },
  { value: SoilCondition.Tl, groupKey: 'clays' },
  { value: SoilCondition.FS, groupKey: 'pureSands' },
  { value: SoilCondition.MS, groupKey: 'pureSands' },
  { value: SoilCondition.GS, groupKey: 'pureSands' },
  { value: SoilCondition.Unknown, groupKey: 'other' },
]

/**
 * Reactive to language change: re-renders whichever component calls it.
 * Falls back to the raw value for anything outside the KA5 catalog (e.g. an
 * absent soil condition), matching the pre-catalog lookup's `?? value`.
 */
export const useSoilConditionLabel = (): ((value: SoilCondition) => string) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (value: SoilCondition) =>
      SoilConditionEntries.some((entry) => entry.value === value)
        ? translate(`soilCondition.${value}.label`)
        : value,
    [translate],
  )
}

/** The full KA5 option list with translated labels and group headings, in display order. */
export const useSoilConditionOptions = (): SoilConditionOption[] => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useMemo(
    () =>
      SoilConditionEntries.map(({ value, groupKey }) => ({
        value,
        label: translate(`soilCondition.${value}.label`),
        group: translate(`soilCondition.groups.${groupKey}`),
      })),
    [translate],
  )
}
