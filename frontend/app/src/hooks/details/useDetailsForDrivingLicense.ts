import { DrivingLicense } from '@green-ecolution/backend-client'

export const DrivingLicenseOptions = [
  {
    value: DrivingLicense.DrivingLicenseB,
    label: 'B',
  },
  {
    value: DrivingLicense.DrivingLicenseBE,
    label: 'BE',
  },
  {
    value: DrivingLicense.DrivingLicenseC,
    label: 'C',
  },
  {
    value: DrivingLicense.DrivingLicenseCE,
    label: 'CE',
  },
]

export const getDrivingLicenseDetails = (drivingLicense: DrivingLicense) =>
  DrivingLicenseOptions.find((option) => option.value === drivingLicense) ??
  DrivingLicenseOptions[0]

export const parseDrivingLicense = (drivingLicense: string): DrivingLicense => {
  switch (drivingLicense) {
    case 'B':
      return DrivingLicense.DrivingLicenseB
    case 'BE':
      return DrivingLicense.DrivingLicenseBE
    case 'C':
      return DrivingLicense.DrivingLicenseC
    case 'CE':
      return DrivingLicense.DrivingLicenseCE
    default:
      return DrivingLicense.DrivingLicenseB
  }
}
