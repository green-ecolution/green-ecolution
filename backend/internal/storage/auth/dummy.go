package auth

import (
	"context"
	"slices"

	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

// UserDummyRepo is used to disable the auth service by configuration
type UserDummyRepo struct {
	dummyUsers []*entities.User
}

func NewUserDummyRepo() *UserDummyRepo {
	return &UserDummyRepo{
		dummyUsers: []*entities.User{
			{
				ID:              uuid.New(),
				EmployeeID:      "42",
				FirstName:       "Peter",
				LastName:        "Parser",
				Username:        "pparser",
				Email:           "peter.parser@tbz-flensburg.de",
				EmailVerified:   true,
				DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB, entities.DrivingLicenseBE, entities.DrivingLicenseC},
				Status:          entities.UserStatusAvailable,
				Roles:           []entities.UserRole{entities.UserRoleTbz},
			},
			{
				ID:              uuid.New(),
				EmployeeID:      "187",
				FirstName:       "Julia",
				LastName:        "Jung",
				Username:        "jjung",
				Email:           "julia.jung@tbz-flensburg.de",
				EmailVerified:   true,
				DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB, entities.DrivingLicenseBE, entities.DrivingLicenseC},
				Status:          entities.UserStatusAbsent,
				Roles:           []entities.UserRole{entities.UserRoleTbz},
			},
			{
				ID:              uuid.New(),
				EmployeeID:      "69",
				FirstName:       "Toni",
				LastName:        "Tester",
				Username:        "ttester",
				Email:           "toni.tester@green-ecolution.de",
				EmailVerified:   true,
				DrivingLicenses: []entities.DrivingLicense{entities.DrivingLicenseB, entities.DrivingLicenseBE, entities.DrivingLicenseC, entities.DrivingLicenseCE},
				Status:          entities.UserStatusAvailable,
				Roles:           []entities.UserRole{entities.UserRoleGreenEcolution},
			},
		},
	}
}

func (r *UserDummyRepo) Create(_ context.Context, _ *entities.User, _ string, _ []string) (*entities.User, error) {
	return nil, storage.ErrAuthServiceDisabled
}

func (r *UserDummyRepo) RemoveSession(_ context.Context, _ string) error {
	return nil
}

func (r *UserDummyRepo) GetAll(_ context.Context) ([]*entities.User, error) {
	return r.dummyUsers, nil
}

func (r *UserDummyRepo) GetAllByRole(_ context.Context, role entities.UserRole) ([]*entities.User, error) {
	return utils.Filter(r.dummyUsers, func(u *entities.User) bool {
		return slices.Contains(u.Roles, role)
	}), nil
}

func (r *UserDummyRepo) GetByIDs(_ context.Context, ids []string) ([]*entities.User, error) {
	return utils.Filter(r.dummyUsers, func(u *entities.User) bool {
		return slices.ContainsFunc(ids, func(id string) bool {
			return u.ID.String() == id
		})
	}), nil
}
