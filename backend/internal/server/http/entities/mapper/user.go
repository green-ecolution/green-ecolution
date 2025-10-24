package mapper

import (
	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
// goverter:extend github.com/green-ecolution/backend/internal/utils:UUIDToString
// goverter:extend github.com/green-ecolution/backend/internal/utils:URLToString
// goverter:extend MapDrivingLicense MapUserRoles MapUserStatus
type UserHTTPMapper interface {
	FromResponse(*domain.User) *entities.UserResponse
	FromResponseList([]*domain.User) []*entities.UserResponse
}

func MapUserRoles(userRole domain.UserRole) entities.UserRole {
	return entities.UserRole(userRole)
}

func MapUserStatus(userStatus domain.UserStatus) entities.UserStatus {
	return entities.UserStatus(userStatus)
}
