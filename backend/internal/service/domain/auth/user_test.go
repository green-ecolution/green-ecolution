package auth

import (
	"context"
	"errors"
	"log/slog"
	"net/url"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	storageMock "github.com/green-ecolution/green-ecolution/backend/internal/storage/_mock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

var rootCtx = context.WithValue(context.Background(), "logger", slog.Default())

func TestRegisterUser(t *testing.T) {
	t.Run("should return result when success", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{}
		inputUser := &entities.User{
			Username:    "username",
			Email:       "mail@foo.com",
			PhoneNumber: "phoneNumber",
			FirstName:   "firstName",
			LastName:    "lastName",
			EmployeeID:  "employeeID",
		}
		input := &entities.RegisterUser{
			User:     *inputUser,
			Password: "password",
			Roles:    []string{"viewer"},
		}

		expected := &entities.User{
			ID:            uuid.MustParse("6be4c752-94df-4719-99b1-ce58253eaf75"),
			CreatedAt:     time.Now(),
			Username:      inputUser.Username,
			FirstName:     inputUser.FirstName,
			LastName:      inputUser.LastName,
			Email:         inputUser.Email,
			EmployeeID:    inputUser.EmployeeID,
			PhoneNumber:   inputUser.PhoneNumber,
			EmailVerified: false,
			Avatar:        nil,
		}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		userRepo.EXPECT().Create(rootCtx, inputUser, input.Password, input.Roles).Return(expected, nil)
		resp, err := svc.Register(rootCtx, input)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expected, resp)
	})

}

func TestLoginRequest(t *testing.T) {
	t.Run("should return login url", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{
			OidcProvider: config.OidcProvider{
				BaseURL:    "http://localhost:8080/auth",
				DomainName: "realm",
				AuthURL:    "http://localhost:8080/auth/realms/realm/protocol/openid-connect/auth",
				TokenURL:   "http://localhost:8080/auth/realms/realm/protocol/openid-connect/token",
				Backend: config.OidcClient{
					ClientID:     "backend_client",
					ClientSecret: "backend_secret",
				},
				Frontend: config.OidcClient{
					ClientID:     "frontend_client",
					ClientSecret: "frontend_secret",
				},
			},
		}

		redirectURL, _ := url.Parse("http://localhost:3000/auth/callback")
		loginRequest := &entities.LoginRequest{
			RedirectURL: redirectURL,
		}

		respURL, _ := url.Parse("http://localhost:8080/auth/realms/realm/protocol/openid-connect/auth")
		query := respURL.Query()
		query.Add("client_id", identityConfig.OidcProvider.Frontend.ClientID)
		query.Add("response_type", "code")
		query.Add("redirect_uri", loginRequest.RedirectURL.String())
		respURL.RawQuery = query.Encode()

		expected := &entities.LoginResp{
			LoginURL: respURL,
		}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		resp := svc.LoginRequest(rootCtx, loginRequest)

		// then
		assert.Equal(t, expected, resp)
	})

	t.Run("should panic when failed to parse auth url in config", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{
			OidcProvider: config.OidcProvider{
				AuthURL: "not_a_valid_url",
			},
		}

		loginRequest := &entities.LoginRequest{
			RedirectURL: &url.URL{
				Scheme: "http",
				Host:   "localhost:3000",
				Path:   "/auth/callback",
			},
		}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		assert.Panics(t, func() {
			_ = svc.LoginRequest(rootCtx, loginRequest)
		})
	})
}

func TestClientTokenCallback(t *testing.T) {
	t.Run("should return client token", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{}
		loginCallback := &entities.LoginCallback{
			Code: "code",
			RedirectURL: &url.URL{
				Scheme: "http",
				Host:   "localhost:3000",
				Path:   "/auth/callback",
			},
		}

		expected := &entities.ClientToken{
			AccessToken: "access_token",
		}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		authRepo.EXPECT().GetAccessTokenFromClientCode(rootCtx, loginCallback.Code, loginCallback.RedirectURL.String()).Return(expected, nil)
		resp, err := svc.ClientTokenCallback(rootCtx, loginCallback)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expected, resp)
	})

	t.Run("should return error when failed to get access token", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{}
		loginCallback := &entities.LoginCallback{
			Code: "code",
			RedirectURL: &url.URL{
				Scheme: "http",
				Host:   "localhost:3000",
				Path:   "/auth/callback",
			},
		}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		// when
		authRepo.EXPECT().GetAccessTokenFromClientCode(rootCtx, loginCallback.Code, loginCallback.RedirectURL.String()).Return(nil, assert.AnError)
		_, err := svc.ClientTokenCallback(rootCtx, loginCallback)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to get access token: assert.AnError general error for testing")
	})
}

func TestLogoutRequest(t *testing.T) {
	t.Run("should succeed when logout request is valid", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		logoutRequest := &entities.Logout{RefreshToken: "valid-refresh-token"}
		userRepo.EXPECT().RemoveSession(mock.Anything, logoutRequest.RefreshToken).Return(nil)

		// then
		err := svc.LogoutRequest(rootCtx, logoutRequest)

		// when
		assert.NoError(t, err)
	})

	t.Run("should return error when session removal fails", func(t *testing.T) {
		// given
		identityConfig := &config.IdentityAuthConfig{}

		authRepo := storageMock.NewMockAuthRepository(t)
		userRepo := storageMock.NewMockUserRepository(t)
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		logoutRequest := &entities.Logout{RefreshToken: "valid-refresh-token"}
		userRepo.EXPECT().RemoveSession(mock.Anything, logoutRequest.RefreshToken).Return(errors.New(""))

		// when
		err := svc.LogoutRequest(rootCtx, logoutRequest)

		// then
		assert.Error(t, err)
		// assert.EqualError(t, err, "500: failed to remove user session: ")
	})
}

func TestGetAllUsers(t *testing.T) {
	t.Run("should return all users successfully", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)
		uuid01, _ := uuid.NewRandom()
		uuid02, _ := uuid.NewRandom()

		expectedUsers := []*entities.User{
			{
				ID:          uuid01,
				Username:    "user1",
				FirstName:   "John",
				LastName:    "Doe",
				Email:       "john.doe@example.com",
				PhoneNumber: "+123456789",
			},
			{
				ID:          uuid02,
				Username:    "user2",
				FirstName:   "Jane",
				LastName:    "Smith",
				Email:       "jane.smith@example.com",
				PhoneNumber: "+987654321",
			},
		}

		userRepo.EXPECT().GetAll(rootCtx).Return(expectedUsers, nil)

		// when
		users, err := svc.GetAll(rootCtx)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedUsers, users)
	})

	t.Run("should return error when user repository fails", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		userRepo.EXPECT().GetAll(rootCtx).Return(nil, errors.New("repository error"))

		// when
		users, err := svc.GetAll(rootCtx)

		// then
		assert.Error(t, err)
		assert.Nil(t, users)
		// assert.Contains(t, err.Error(), "failed to get all users")
	})

	t.Run("should return empty slice when no users are found", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		userRepo.EXPECT().GetAll(rootCtx).Return([]*entities.User{}, nil)

		// when
		users, err := svc.GetAll(rootCtx)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, users)
		assert.Empty(t, users)
	})
}

func TestGetByIDs(t *testing.T) {
	uuid01, _ := uuid.NewRandom()
	uuid02, _ := uuid.NewRandom()
	input := []string{uuid01.String(), uuid02.String()}

	expectedUsers := []*entities.User{
		{
			ID:          uuid01,
			Username:    "user1",
			FirstName:   "John",
			LastName:    "Doe",
			Email:       "john.doe@example.com",
			PhoneNumber: "+123456789",
		},
		{
			ID:          uuid02,
			Username:    "user2",
			FirstName:   "Jane",
			LastName:    "Smith",
			Email:       "jane.smith@example.com",
			PhoneNumber: "+987654321",
		},
	}
	t.Run("should return all users by ids successfully", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)
		userRepo.EXPECT().GetByIDs(rootCtx, input).Return(expectedUsers, nil)

		// when
		users, err := svc.GetByIDs(rootCtx, input)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedUsers, users)
	})

	t.Run("should return error when user repository fails", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		userRepo.EXPECT().GetByIDs(rootCtx, input).Return(nil, errors.New("repository error"))

		// when
		users, err := svc.GetByIDs(rootCtx, input)

		// then
		assert.Error(t, err)
		assert.Nil(t, users)
		// assert.Contains(t, err.Error(), "failed to get users by ids")
	})

	t.Run("should return empty slice when no users are found", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		userRepo.EXPECT().GetByIDs(rootCtx, input).Return([]*entities.User{}, nil)

		// when
		users, err := svc.GetByIDs(rootCtx, input)

		// then
		assert.NoError(t, err)
		assert.NotNil(t, users)
		assert.Empty(t, users)
	})
}

func TestGetAllByRole(t *testing.T) {
	t.Run("should return users matching the role", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		uuid01, _ := uuid.NewRandom()
		uuid02, _ := uuid.NewRandom()

		expectedRole := entities.UserRoleTbz
		expectedUsers := []*entities.User{
			{
				ID:          uuid01,
				Username:    "admin1",
				FirstName:   "John",
				LastName:    "Doe",
				Email:       "admin1@example.com",
				PhoneNumber: "+123456789",
				Roles:       []entities.UserRole{entities.UserRoleTbz},
			},
			{
				ID:          uuid02,
				Username:    "admin2",
				FirstName:   "Jane",
				LastName:    "Smith",
				Email:       "admin2@example.com",
				PhoneNumber: "+987654321",
				Roles:       []entities.UserRole{entities.UserRoleTbz},
			},
		}

		userRepo.EXPECT().GetAllByRole(rootCtx, expectedRole).Return(expectedUsers, nil)

		// when
		users, err := svc.GetAllByRole(rootCtx, expectedRole)

		// then
		assert.NoError(t, err)
		assert.Equal(t, expectedUsers, users)
	})

	t.Run("should return empty slice when no users match the role", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		expectedRole := entities.UserRoleTbz
		userRepo.EXPECT().GetAllByRole(rootCtx, expectedRole).Return([]*entities.User{}, nil)

		// when
		users, err := svc.GetAllByRole(rootCtx, expectedRole)

		// then
		assert.NoError(t, err)
		assert.Empty(t, users)
	})

	t.Run("should return error when underlying repository fails", func(t *testing.T) {
		// given
		userRepo := storageMock.NewMockUserRepository(t)
		authRepo := storageMock.NewMockAuthRepository(t)
		identityConfig := &config.IdentityAuthConfig{}
		svc := NewAuthService(authRepo, userRepo, identityConfig)

		userRepo.EXPECT().GetAllByRole(rootCtx, entities.UserRoleTbz).Return(nil, errors.New("repository error"))

		// when
		users, err := svc.GetAllByRole(rootCtx, entities.UserRoleTbz)

		// then
		assert.Error(t, err)
		assert.Nil(t, users)
		// assert.Contains(t, err.Error(), "repository error")
	})
}
