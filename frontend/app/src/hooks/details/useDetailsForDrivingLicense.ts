import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { DrivingLicense } from '@green-ecolution/backend-client'
import { createEnumParser } from '@/lib/enumLookup'

// `t`'s generated overloads only accept the catalog's literal key union; the
// enum value plugged into the template isn't statically one of those literals.
type EnumsTranslate = (key: string) => string

const DrivingLicenseValues: DrivingLicense[] = [
  DrivingLicense.B,
  DrivingLicense.Be,
  DrivingLicense.C,
  DrivingLicense.Ce,
]

export interface DrivingLicenseDetails {
  value: DrivingLicense
  label: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useDrivingLicenseDetails = (): ((
  license: DrivingLicense,
) => DrivingLicenseDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (license: DrivingLicense): DrivingLicenseDetails => ({
      value: license,
      label: translate(`drivingLicense.${license}.label`),
    }),
    [translate],
  )
}

/** The full class list with translated labels, in display order. */
export const useDrivingLicenseOptions = (): DrivingLicenseDetails[] => {
  const getDetails = useDrivingLicenseDetails()
  return useMemo(() => DrivingLicenseValues.map(getDetails), [getDetails])
}

export const parseDrivingLicense = createEnumParser<DrivingLicense>(
  {
    B: DrivingLicense.B,
    BE: DrivingLicense.Be,
    C: DrivingLicense.C,
    CE: DrivingLicense.Ce,
  },
  DrivingLicense.B,
)
