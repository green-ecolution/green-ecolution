package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

func UserFromResponse(source *domain.User) *entities.UserResponse {
	if source == nil {
		return nil
	}
	resp := &entities.UserResponse{
		ID:            utils.UUIDToString(source.ID),
		CreatedAt:     source.CreatedAt,
		Username:      source.Username,
		FirstName:     source.FirstName,
		LastName:      source.LastName,
		Email:         source.Email,
		EmployeeID:    source.EmployeeID,
		PhoneNumber:   source.PhoneNumber,
		EmailVerified: source.EmailVerified,
		Avatar:        utils.URLToString(source.Avatar),
		Status:        MapUserStatus(source.Status),
	}
	if source.Roles != nil {
		resp.Roles = make([]entities.UserRole, len(source.Roles))
		for i, r := range source.Roles {
			resp.Roles[i] = MapUserRoles(r)
		}
	}
	if source.DrivingLicenses != nil {
		resp.DrivingLicenses = make([]entities.DrivingLicense, len(source.DrivingLicenses))
		for i, d := range source.DrivingLicenses {
			resp.DrivingLicenses[i] = MapDrivingLicense(d)
		}
	}
	return resp
}

func UserFromResponseList(source []*domain.User) []*entities.UserResponse {
	return utils.MapSlice(source, UserFromResponse)
}

func MapUserRoles(userRole domain.UserRole) entities.UserRole {
	return entities.UserRole(userRole)
}

func MapUserStatus(userStatus domain.UserStatus) entities.UserStatus {
	return entities.UserStatus(userStatus)
}
