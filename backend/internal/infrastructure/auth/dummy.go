package auth

import (
	"context"
	"slices"

	"github.com/google/uuid"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/user"
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/vehicle"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// UserDummyRepo is used to disable the auth service by configuration
type UserDummyRepo struct {
	dummyUsers []*user.User
}

func NewUserDummyRepo() *UserDummyRepo {
	return &UserDummyRepo{
		dummyUsers: []*user.User{
			{
				ID:              uuid.New(),
				EmployeeID:      "42",
				FirstName:       "Peter",
				LastName:        "Parser",
				Username:        "pparser",
				Email:           "peter.parser@tbz-flensburg.de",
				EmailVerified:   true,
				DrivingLicenses: []vehicle.DrivingLicense{vehicle.DrivingLicenseB, vehicle.DrivingLicenseBE, vehicle.DrivingLicenseC},
				Status:          user.UserStatusAvailable,
				Roles:           []user.UserRole{user.UserRoleTbz},
			},
			{
				ID:              uuid.New(),
				EmployeeID:      "187",
				FirstName:       "Julia",
				LastName:        "Jung",
				Username:        "jjung",
				Email:           "julia.jung@tbz-flensburg.de",
				EmailVerified:   true,
				DrivingLicenses: []vehicle.DrivingLicense{vehicle.DrivingLicenseB, vehicle.DrivingLicenseBE, vehicle.DrivingLicenseC},
				Status:          user.UserStatusAbsent,
				Roles:           []user.UserRole{user.UserRoleTbz},
			},
			{
				ID:              uuid.New(),
				EmployeeID:      "69",
				FirstName:       "Toni",
				LastName:        "Tester",
				Username:        "ttester",
				Email:           "toni.tester@green-ecolution.de",
				EmailVerified:   true,
				DrivingLicenses: []vehicle.DrivingLicense{vehicle.DrivingLicenseB, vehicle.DrivingLicenseBE, vehicle.DrivingLicenseC, vehicle.DrivingLicenseCE},
				Status:          user.UserStatusAvailable,
				Roles:           []user.UserRole{user.UserRoleGreenEcolution},
			},
		},
	}
}

func (r *UserDummyRepo) Create(_ context.Context, _ *user.User, _ string, _ []string) (*user.User, error) {
	return nil, shared.ErrAuthServiceDisabled
}

func (r *UserDummyRepo) RemoveSession(_ context.Context, _ string) error {
	return nil
}

func (r *UserDummyRepo) GetAll(_ context.Context) ([]*user.User, error) {
	return r.dummyUsers, nil
}

func (r *UserDummyRepo) GetAllByRole(_ context.Context, role user.UserRole) ([]*user.User, error) {
	return utils.Filter(r.dummyUsers, func(u *user.User) bool {
		return slices.Contains(u.Roles, role)
	}), nil
}

func (r *UserDummyRepo) GetByIDs(_ context.Context, ids []string) ([]*user.User, error) {
	return utils.Filter(r.dummyUsers, func(u *user.User) bool {
		return slices.ContainsFunc(ids, func(id string) bool {
			return u.ID.String() == id
		})
	}), nil
}
