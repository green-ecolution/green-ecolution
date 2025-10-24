package entities

type DrivingLicense string

const (
	DrivingLicenseB  DrivingLicense = "B"
	DrivingLicenseBE DrivingLicense = "BE"
	DrivingLicenseC  DrivingLicense = "C"
	DrivingLicenseCE DrivingLicense = "CE"
)

func ParseDrivingLicense(drivingLicense string) DrivingLicense {
	switch drivingLicense {
	case string(DrivingLicenseB):
		return DrivingLicenseB
	case string(DrivingLicenseBE):
		return DrivingLicenseBE
	case string(DrivingLicenseC):
		return DrivingLicenseC
	case string(DrivingLicenseCE):
		return DrivingLicenseCE
	default:
		return DrivingLicenseB
	}
}
