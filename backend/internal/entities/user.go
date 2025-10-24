package entities

import (
	"net/url"
	"time"

	"github.com/google/uuid"
)

type UserStatus string

const (
	UserStatusAvailable UserStatus = "available"
	UserStatusAbsent    UserStatus = "absent"
	UserStatusUnknown   UserStatus = "unknown"
)

type UserRole string

const (
	UserRoleTbz               UserRole = "tbz"
	UserRoleGreenEcolution    UserRole = "green-ecolution"
	UserRoleSmarteGrenzregion UserRole = "smarte-grenzregion"
	UserRoleUnknown           UserRole = "unknown"
)

type User struct {
	ID              uuid.UUID
	CreatedAt       time.Time
	Username        string `validate:"required,min=3,max=15"`
	FirstName       string `validate:"required,min=3,max=30"`
	LastName        string `validate:"required,min=3,max=30"`
	Email           string `validate:"required,email"`
	EmployeeID      string
	PhoneNumber     string
	EmailVerified   bool
	Roles           []UserRole
	Avatar          *url.URL
	DrivingLicenses []DrivingLicense
	Status          UserStatus
}

type RegisterUser struct {
	User     User
	Password string `validate:"required"`
	Roles    []string
}

func ParseUserStatus(status string) UserStatus {
	switch status {
	case string(UserStatusAvailable):
		return UserStatusAvailable
	case string(UserStatusAbsent):
		return UserStatusAbsent
	default:
		return UserStatusUnknown
	}
}

func ParseUserRole(role string) UserRole {
	switch role {
	case string(UserRoleTbz):
		return UserRoleTbz
	case string(UserRoleGreenEcolution):
		return UserRoleGreenEcolution
	case string(UserRoleSmarteGrenzregion):
		return UserRoleSmarteGrenzregion
	default:
		return UserRoleUnknown
	}
}
