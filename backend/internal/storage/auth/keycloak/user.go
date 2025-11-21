package keycloak

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/Nerzal/gocloak/v13"
	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type UserRepository struct {
	cfg *config.IdentityAuthConfig
}

func NewUserRepository(cfg *config.IdentityAuthConfig) storage.UserRepository {
	return &UserRepository{
		cfg: cfg,
	}
}

func (r *UserRepository) Create(ctx context.Context, user *entities.User, password string, roles []string) (*entities.User, error) {
	log := logger.GetLogger(ctx)
	if user == nil {
		return nil, ErrEmptyUser
	}

	log.Debug("creating user in keycloak", "user_name", user.Username)
	log.Debug("creating user in keycloak with detailed information", "roles", roles, "raw_user", fmt.Sprintf("%+v", user))

	keyCloakUser := userToKeyCloakUser(user)

	client, clientToken, err := loginRestAPIClient(ctx, r.cfg.OidcProvider.BaseURL, r.cfg.OidcProvider.Backend.ClientID, r.cfg.OidcProvider.Backend.ClientSecret, r.cfg.OidcProvider.DomainName)
	if err != nil {
		return nil, err
	}

	userID, err := client.CreateUser(ctx, clientToken.AccessToken, r.cfg.OidcProvider.DomainName, *keyCloakUser)
	if err != nil {
		log.Error("failed to generate user in keycloak", "error", err, "user_id", userID)
		return nil, errors.Join(err, ErrCreateUser)
	}

	if err = client.SetPassword(ctx, clientToken.AccessToken, userID, r.cfg.OidcProvider.DomainName, password, false); err != nil {
		log.Error("failed to set password of currently created user", "error", err, "user_id", userID)
		return nil, errors.Join(err, ErrSetPassword)
	}

	if roles != nil || len(roles) > 0 {
		kcRoles := make([]gocloak.Role, len(roles))
		for i, roleName := range roles {
			var roleKeyCloak *gocloak.Role
			roleNameLowerCase := strings.ToLower(roleName)
			roleKeyCloak, err = client.GetRealmRole(ctx, clientToken.AccessToken, r.cfg.OidcProvider.DomainName, roleNameLowerCase)
			if err != nil {
				log.Error("failed to get role by name", "role", roleNameLowerCase, "error", err)
				return nil, errors.Join(err, ErrGetRole)
			}

			if roleKeyCloak != nil {
				kcRoles[i] = *roleKeyCloak
			}
		}
		if err = client.AddRealmRoleToUser(ctx, clientToken.AccessToken, r.cfg.OidcProvider.DomainName, userID, kcRoles); err != nil {
			log.Error("failed to assign roles to user", "role", kcRoles, "error", err)
			return nil, errors.Join(err, ErrSetRole)
		}
	}

	userKeyCloak, err := client.GetUserByID(ctx, clientToken.AccessToken, r.cfg.OidcProvider.DomainName, userID)
	if err != nil {
		log.Error("failed to get recently created user by id", "user_id", userID, "error", err)
		return nil, errors.Join(err, ErrGetUser)
	}

	log.Debug("user created successfully", "user_id", *userKeyCloak.ID)

	return keyCloakUserToUser(ctx, userKeyCloak)
}

func (r *UserRepository) RemoveSession(ctx context.Context, refreshToken string) error {
	log := logger.GetLogger(ctx)
	client := gocloak.NewClient(r.cfg.OidcProvider.BaseURL)
	if err := client.Logout(ctx, r.cfg.OidcProvider.Frontend.ClientID, r.cfg.OidcProvider.Frontend.ClientSecret, r.cfg.OidcProvider.DomainName, refreshToken); err != nil {
		log.Error("failed to remove user session from keycloak", "error", err, "refresh_token", refreshToken)
		return errors.Join(err, ErrLogout)
	}
	return nil
}

func (r *UserRepository) GetAll(ctx context.Context) ([]*entities.User, error) {
	log := logger.GetLogger(ctx)
	client, token, err := loginRestAPIClient(ctx, r.cfg.OidcProvider.BaseURL, r.cfg.OidcProvider.Backend.ClientID, r.cfg.OidcProvider.Backend.ClientSecret, r.cfg.OidcProvider.DomainName)
	if err != nil {
		return nil, err
	}

	users, err := client.GetUsers(ctx, token.AccessToken, r.cfg.OidcProvider.DomainName, gocloak.GetUsersParams{})
	if err != nil {
		log.Error("failed to get user from keycloak", "error", err)
		return nil, errors.Join(err, ErrGetUser)
	}

	allUsers := make([]*entities.User, len(users))
	for i, kcUser := range users {
		user, err := keyCloakUserToUser(ctx, kcUser)
		if err != nil && !errors.Is(err, ErrUserWithNilAttributes) { // skip users without required attributes
			return nil, err
		}
		if user != nil {
			allUsers[i] = user
		}
	}

	return allUsers, nil
}

func (r *UserRepository) GetAllByRole(ctx context.Context, role entities.UserRole) ([]*entities.User, error) {
	log := logger.GetLogger(ctx)
	users, err := r.GetAll(ctx)
	if err != nil {
		log.Debug("failed to fetch all users", "error", err)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return utils.Filter(users, func(user *entities.User) bool {
		for _, userRole := range user.Roles {
			if userRole == role {
				return true
			}
		}
		return false
	}), nil
}

func (r *UserRepository) GetByIDs(ctx context.Context, ids []string) ([]*entities.User, error) {
	log := logger.GetLogger(ctx)
	client, token, err := loginRestAPIClient(ctx, r.cfg.OidcProvider.BaseURL, r.cfg.OidcProvider.Backend.ClientID, r.cfg.OidcProvider.Backend.ClientSecret, r.cfg.OidcProvider.DomainName)
	if err != nil {
		return nil, err
	}

	users := make([]*entities.User, len(ids))
	for i, id := range ids {
		kcUser, err := client.GetUserByID(ctx, token.AccessToken, r.cfg.OidcProvider.DomainName, id)
		if err != nil {
			log.Error("failed to get user by id", "error", err, "user_id", id)
			return nil, err
		}

		user, err := keyCloakUserToUser(ctx, kcUser)
		if err != nil {
			return nil, err
		}
		users[i] = user
	}

	return users, nil
}

func keyCloakUserToUser(ctx context.Context, user *gocloak.User) (*entities.User, error) {
	log := logger.GetLogger(ctx)
	userID, err := uuid.Parse(*user.ID)
	if err != nil {
		log.Error("failed to parse user id", "user_id", *user.ID, "err", err)
		return nil, ErrParseID
	}

	if err := validateRequiredAttributes(user); err != nil {
		return nil, err
	}

	var phoneNumber, employeeID, status string
	var userRoles, drivingLicenses []string
	if user.Attributes != nil {
		if val, ok := (*user.Attributes)["phone_number"]; ok && len(val) > 0 {
			phoneNumber = val[0]
		}

		if val, ok := (*user.Attributes)["employee_id"]; ok && len(val) > 0 {
			employeeID = val[0]
		}

		if val, ok := (*user.Attributes)["driving_licenses"]; ok && len(val) > 0 {
			drivingLicenses = val
		}

		if val, ok := (*user.Attributes)["user_roles"]; ok && len(val) > 0 {
			userRoles = val
		}

		if val, ok := (*user.Attributes)["status"]; ok && len(val) > 0 {
			status = val[0]
		}
	}

	roles := convertRoles(userRoles)
	lisences := convertDrivingLicenses(drivingLicenses)

	const millisecondsInSecond = 1000
	return &entities.User{
		ID:              userID,
		CreatedAt:       time.Unix(*user.CreatedTimestamp/millisecondsInSecond, 0),
		Username:        *user.Username,
		FirstName:       *user.FirstName,
		LastName:        *user.LastName,
		Email:           *user.Email,
		PhoneNumber:     phoneNumber,
		EmployeeID:      employeeID,
		Roles:           roles,
		DrivingLicenses: lisences,
		Status:          entities.ParseUserStatus(status),
	}, nil
}

func convertRoles(userRoles []string) []entities.UserRole {
	if userRoles == nil {
		return []entities.UserRole{}
	}

	var roles []entities.UserRole
	for _, roleName := range userRoles {
		userRole := entities.ParseUserRole(roleName)
		roles = append(roles, userRole)
	}
	return roles
}

func convertDrivingLicenses(drivingLicenses []string) []entities.DrivingLicense {
	if drivingLicenses == nil {
		return []entities.DrivingLicense{}
	}

	var licenses []entities.DrivingLicense
	for _, drivingLicense := range drivingLicenses {
		license := entities.ParseDrivingLicense(drivingLicense)
		licenses = append(licenses, license)
	}

	return licenses
}

func validateRequiredAttributes(user *gocloak.User) error {
	if user.FirstName == nil || user.Email == nil || user.LastName == nil || user.Username == nil {
		return ErrUserWithNilAttributes
	}
	return nil
}

func userToKeyCloakUser(user *entities.User) *gocloak.User {
	attribute := make(map[string][]string)
	attribute["phone_number"] = []string{user.PhoneNumber}
	attribute["employee_id"] = []string{user.EmployeeID}

	return &gocloak.User{
		ID:         gocloak.StringP(user.ID.String()),
		Username:   gocloak.StringP(user.Username),
		FirstName:  gocloak.StringP(user.FirstName),
		LastName:   gocloak.StringP(user.LastName),
		Email:      gocloak.StringP(user.Email),
		Enabled:    gocloak.BoolP(true),
		Attributes: &attribute,
	}
}
