package entities

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestUser_HasRole(t *testing.T) {
	user := &User{Roles: []UserRole{UserRoleTbz, UserRoleGreenEcolution}}

	assert.True(t, user.HasRole(UserRoleTbz))
	assert.True(t, user.HasRole(UserRoleGreenEcolution))
	assert.False(t, user.HasRole(UserRoleSmarteGrenzregion))
	assert.False(t, (&User{}).HasRole(UserRoleTbz))
}

func TestUser_HasRequiredLicenses(t *testing.T) {
	t.Run("CE user satisfies transporter C and trailer BE", func(t *testing.T) {
		user := &User{DrivingLicenses: []DrivingLicense{DrivingLicenseCE}}
		assert.True(t, user.HasRequiredLicenses([]DrivingLicense{DrivingLicenseC, DrivingLicenseBE}))
	})

	t.Run("B user does not satisfy transporter C and trailer BE", func(t *testing.T) {
		user := &User{DrivingLicenses: []DrivingLicense{DrivingLicenseB}}
		assert.False(t, user.HasRequiredLicenses([]DrivingLicense{DrivingLicenseC, DrivingLicenseBE}))
	})

	t.Run("C user satisfies transporter B only", func(t *testing.T) {
		user := &User{DrivingLicenses: []DrivingLicense{DrivingLicenseC}}
		assert.True(t, user.HasRequiredLicenses([]DrivingLicense{DrivingLicenseB}))
	})

	t.Run("empty required licenses always passes", func(t *testing.T) {
		user := &User{DrivingLicenses: []DrivingLicense{DrivingLicenseB}}
		assert.True(t, user.HasRequiredLicenses([]DrivingLicense{}))
	})

	t.Run("user with no licenses fails", func(t *testing.T) {
		user := &User{}
		assert.False(t, user.HasRequiredLicenses([]DrivingLicense{DrivingLicenseB}))
	})
}
