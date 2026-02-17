package wateringplan

import (
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
)

func TestLicenseSatisfies(t *testing.T) {
	tests := []struct {
		name     string
		held     entities.DrivingLicense
		required entities.DrivingLicense
		want     bool
	}{
		// B satisfies only B
		{"B satisfies B", entities.DrivingLicenseB, entities.DrivingLicenseB, true},
		{"B does not satisfy BE", entities.DrivingLicenseB, entities.DrivingLicenseBE, false},
		{"B does not satisfy C", entities.DrivingLicenseB, entities.DrivingLicenseC, false},
		{"B does not satisfy CE", entities.DrivingLicenseB, entities.DrivingLicenseCE, false},

		// BE satisfies B, BE
		{"BE satisfies B", entities.DrivingLicenseBE, entities.DrivingLicenseB, true},
		{"BE satisfies BE", entities.DrivingLicenseBE, entities.DrivingLicenseBE, true},
		{"BE does not satisfy C", entities.DrivingLicenseBE, entities.DrivingLicenseC, false},
		{"BE does not satisfy CE", entities.DrivingLicenseBE, entities.DrivingLicenseCE, false},

		// C satisfies B, C
		{"C satisfies B", entities.DrivingLicenseC, entities.DrivingLicenseB, true},
		{"C satisfies C", entities.DrivingLicenseC, entities.DrivingLicenseC, true},
		{"C does not satisfy BE", entities.DrivingLicenseC, entities.DrivingLicenseBE, false},
		{"C does not satisfy CE", entities.DrivingLicenseC, entities.DrivingLicenseCE, false},

		// CE satisfies B, BE, C, CE
		{"CE satisfies B", entities.DrivingLicenseCE, entities.DrivingLicenseB, true},
		{"CE satisfies BE", entities.DrivingLicenseCE, entities.DrivingLicenseBE, true},
		{"CE satisfies C", entities.DrivingLicenseCE, entities.DrivingLicenseC, true},
		{"CE satisfies CE", entities.DrivingLicenseCE, entities.DrivingLicenseCE, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := licenseSatisfies(tt.held, tt.required)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestHasValidLicense(t *testing.T) {
	t.Run("user with C license satisfies B requirement", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseC},
		}
		assert.True(t, hasValidLicense(user, entities.DrivingLicenseB))
	})

	t.Run("user with B license does not satisfy C requirement", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB},
		}
		assert.False(t, hasValidLicense(user, entities.DrivingLicenseC))
	})

	t.Run("user with multiple licenses satisfies if any matches", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB, entities.DrivingLicenseBE},
		}
		assert.True(t, hasValidLicense(user, entities.DrivingLicenseBE))
	})

	t.Run("user with no licenses does not satisfy any requirement", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{},
		}
		assert.False(t, hasValidLicense(user, entities.DrivingLicenseB))
	})
}

func TestHasAllRequiredLicenses(t *testing.T) {
	t.Run("CE user satisfies transporter C and trailer BE", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseCE},
		}
		required := []entities.DrivingLicense{entities.DrivingLicenseC, entities.DrivingLicenseBE}
		assert.True(t, hasAllRequiredLicenses(user, required))
	})

	t.Run("B user does not satisfy transporter C and trailer BE", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB},
		}
		required := []entities.DrivingLicense{entities.DrivingLicenseC, entities.DrivingLicenseBE}
		assert.False(t, hasAllRequiredLicenses(user, required))
	})

	t.Run("C user satisfies transporter B only", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseC},
		}
		required := []entities.DrivingLicense{entities.DrivingLicenseB}
		assert.True(t, hasAllRequiredLicenses(user, required))
	})

	t.Run("empty required licenses always passes", func(t *testing.T) {
		user := &entities.User{
			DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB},
		}
		assert.True(t, hasAllRequiredLicenses(user, []entities.DrivingLicense{}))
	})
}
