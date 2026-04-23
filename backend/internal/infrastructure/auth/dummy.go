package auth

import (
	"context"
	"slices"

	"github.com/google/uuid"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// UserDummyRepo is used to disable the auth service by configuration
type UserDummyRepo struct {
	dummyUsers []*shared.User
}

func NewUserDummyRepo() *UserDummyRepo {
	return &UserDummyRepo{
		dummyUsers: []*shared.User{
			{
				ID:              uuid.New(),
				EmployeeID:      "42",
				FirstName:       "Peter",
				LastName:        "Parser",
				Username:        "pparser",
				Email:           "peter.parser@tbz-flensburg.de",
				EmailVerified:   true,
				DrivingLicenses: []shared.DrivingLicense{shared.DrivingLicenseB, shared.DrivingLicenseBE, shared.DrivingLicenseC},
				Status:          shared.UserStatusAvailable,
				Roles:           []shared.UserRole{shared.UserRoleTbz},
			},
			{
				ID:              uuid.New(),
				EmployeeID:      "187",
				FirstName:       "Julia",
				LastName:        "Jung",
				Username:        "jjung",
				Email:           "julia.jung@tbz-flensburg.de",
				EmailVerified:   true,
				DrivingLicenses: []shared.DrivingLicense{shared.DrivingLicenseB, shared.DrivingLicenseBE, shared.DrivingLicenseC},
				Status:          shared.UserStatusAbsent,
				Roles:           []shared.UserRole{shared.UserRoleTbz},
			},
			{
				ID:              uuid.New(),
				EmployeeID:      "69",
				FirstName:       "Toni",
				LastName:        "Tester",
				Username:        "ttester",
				Email:           "toni.tester@green-ecolution.de",
				EmailVerified:   true,
				DrivingLicenses: []shared.DrivingLicense{shared.DrivingLicenseB, shared.DrivingLicenseBE, shared.DrivingLicenseC, shared.DrivingLicenseCE},
				Status:          shared.UserStatusAvailable,
				Roles:           []shared.UserRole{shared.UserRoleGreenEcolution},
			},
		},
	}
}

func (r *UserDummyRepo) Create(_ context.Context, _ *shared.User, _ string, _ []string) (*shared.User, error) {
	return nil, shared.ErrAuthServiceDisabled
}

func (r *UserDummyRepo) RemoveSession(_ context.Context, _ string) error {
	return nil
}

func (r *UserDummyRepo) GetAll(_ context.Context) ([]*shared.User, error) {
	return r.dummyUsers, nil
}

func (r *UserDummyRepo) GetAllByRole(_ context.Context, role shared.UserRole) ([]*shared.User, error) {
	return utils.Filter(r.dummyUsers, func(u *shared.User) bool {
		return slices.Contains(u.Roles, role)
	}), nil
}

func (r *UserDummyRepo) GetByIDs(_ context.Context, ids []string) ([]*shared.User, error) {
	return utils.Filter(r.dummyUsers, func(u *shared.User) bool {
		return slices.ContainsFunc(ids, func(id string) bool {
			return u.ID.String() == id
		})
	}), nil
}
