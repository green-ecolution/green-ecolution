package entities

import "fmt"

type DrivingLicense string

const (
	DrivingLicenseB  DrivingLicense = "B"
	DrivingLicenseBE DrivingLicense = "BE"
	DrivingLicenseC  DrivingLicense = "C"
	DrivingLicenseCE DrivingLicense = "CE"
)

func ParseDrivingLicense(drivingLicense string) (DrivingLicense, error) {
	switch drivingLicense {
	case string(DrivingLicenseB):
		return DrivingLicenseB, nil
	case string(DrivingLicenseBE):
		return DrivingLicenseBE, nil
	case string(DrivingLicenseC):
		return DrivingLicenseC, nil
	case string(DrivingLicenseCE):
		return DrivingLicenseCE, nil
	default:
		return "", fmt.Errorf("invalid driving license: %q", drivingLicense)
	}
}
