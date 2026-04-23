package entities

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDrivingLicense_Satisfies(t *testing.T) {
	tests := []struct {
		name     string
		held     DrivingLicense
		required DrivingLicense
		want     bool
	}{
		{"B satisfies B", DrivingLicenseB, DrivingLicenseB, true},
		{"B does not satisfy BE", DrivingLicenseB, DrivingLicenseBE, false},
		{"B does not satisfy C", DrivingLicenseB, DrivingLicenseC, false},
		{"B does not satisfy CE", DrivingLicenseB, DrivingLicenseCE, false},

		{"BE satisfies B", DrivingLicenseBE, DrivingLicenseB, true},
		{"BE satisfies BE", DrivingLicenseBE, DrivingLicenseBE, true},
		{"BE does not satisfy C", DrivingLicenseBE, DrivingLicenseC, false},
		{"BE does not satisfy CE", DrivingLicenseBE, DrivingLicenseCE, false},

		{"C satisfies B", DrivingLicenseC, DrivingLicenseB, true},
		{"C satisfies C", DrivingLicenseC, DrivingLicenseC, true},
		{"C does not satisfy BE", DrivingLicenseC, DrivingLicenseBE, false},
		{"C does not satisfy CE", DrivingLicenseC, DrivingLicenseCE, false},

		{"CE satisfies B", DrivingLicenseCE, DrivingLicenseB, true},
		{"CE satisfies BE", DrivingLicenseCE, DrivingLicenseBE, true},
		{"CE satisfies C", DrivingLicenseCE, DrivingLicenseC, true},
		{"CE satisfies CE", DrivingLicenseCE, DrivingLicenseCE, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			assert.Equal(t, tt.want, tt.held.Satisfies(tt.required))
		})
	}
}
