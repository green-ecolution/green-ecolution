import { DrivingLicense } from '@green-ecolution/backend-client'

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

export const getDrivingLicenseDetails = (drivingLicense: DrivingLicense) =>
  DrivingLicenseOptions.find((option) => option.value === drivingLicense) ??
  DrivingLicenseOptions[0]

export const parseDrivingLicense = (drivingLicense: string): DrivingLicense => {
  switch (drivingLicense) {
    case 'B':
      return DrivingLicense.B
    case 'BE':
      return DrivingLicense.Be
    case 'C':
      return DrivingLicense.C
    case 'CE':
      return DrivingLicense.Ce
    default:
      return DrivingLicense.B
  }
}
