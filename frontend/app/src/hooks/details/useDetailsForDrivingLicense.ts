import { DrivingLicense } from '@green-ecolution/backend-client'
import { createEnumLookup, createEnumParser } from '@/lib/enumLookup'

export const DrivingLicenseOptions = [
  {
    value: DrivingLicense.B,
    label: 'B',
  },
  {
    value: DrivingLicense.Be,
    label: 'BE',
  },
  {
    value: DrivingLicense.C,
    label: 'C',
  },
  {
    value: DrivingLicense.Ce,
    label: 'CE',
  },
]

export const getDrivingLicenseDetails = createEnumLookup(DrivingLicenseOptions)

export const parseDrivingLicense = createEnumParser<DrivingLicense>(
  {
    B: DrivingLicense.B,
    BE: DrivingLicense.Be,
    C: DrivingLicense.C,
    CE: DrivingLicense.Ce,
  },
  DrivingLicense.B,
)
