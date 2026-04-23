package entities

import "fmt"

type DrivingLicense string

const (
	DrivingLicenseB  DrivingLicense = "B"
	DrivingLicenseBE DrivingLicense = "BE"
	DrivingLicenseC  DrivingLicense = "C"
	DrivingLicenseCE DrivingLicense = "CE"
)

// Satisfies checks if this license covers the required license (hierarchical).
// Keep in sync with frontend: frontend/app/src/lib/licenseValidation.ts
func (dl DrivingLicense) Satisfies(required DrivingLicense) bool {
	if dl == required {
		return true
	}
	switch dl {
	case DrivingLicenseBE:
		return required == DrivingLicenseB
	case DrivingLicenseC:
		return required == DrivingLicenseB
	case DrivingLicenseCE:
		return required == DrivingLicenseB ||
			required == DrivingLicenseBE ||
			required == DrivingLicenseC
	}
	return false
}

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
