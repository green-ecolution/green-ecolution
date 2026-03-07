package user

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	serviceMock "github.com/green-ecolution/green-ecolution/backend/internal/service/_mock"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/pkg/errors"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
)

func TestLogin(t *testing.T) {
	t.Run("should login user sucessfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/login", Login(mockAuthService))

		parsedURLRedirect, _ := url.Parse("http://example.com/redirect")
		parsedURLResponse, _ := url.Parse("http://example.com/login")

		loginRequest := &domain.LoginRequest{
			RedirectURL: parsedURLRedirect,
		}
		loginResponse := &domain.LoginResp{
			LoginURL: parsedURLResponse,
		}
		mockAuthService.EXPECT().LoginRequest(mock.Anything, loginRequest).Return(loginResponse)

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user/login?redirect_url="+parsedURLRedirect.String(), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 400 bad request for invalid request body.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/login", Login(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user/login", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 400 for invalid redirect url.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/login", Login(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user/login?redirect_url=invalid-url", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})
}

func TestLogout(t *testing.T) {
	t.Run("should logout user sucessfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/logout", Logout(mockAuthService))

		domainEntity := domain.Logout{
			RefreshToken: "valid_refresh_token",
		}

		// when
		mockAuthService.EXPECT().LogoutRequest(mock.Anything, &domainEntity).Return(nil)
		reqBody, _ := json.Marshal(entities.LogoutRequest{
			RefreshToken: "valid_refresh_token",
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/logout", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 400 bad request for invalid request body.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/logout", Logout(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodPost, "/v1/user/logout", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 500 for invalid logout.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/logout", Logout(mockAuthService))

		domainEntity := domain.Logout{
			RefreshToken: "valid_refresh_token",
		}

		// when
		mockAuthService.EXPECT().LogoutRequest(mock.Anything, &domainEntity).Return(errors.New("service error"))
		reqBody, _ := json.Marshal(entities.LogoutRequest{
			RefreshToken: "valid_refresh_token",
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/logout", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})
}

func TestRequestToken(t *testing.T) {
	t.Run("should request token sucessfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/token", RequestToken(mockAuthService))

		redirectURL, _ := url.Parse("http://example.com/redirect")
		domainEntity := domain.LoginCallback{
			Code:        "valid_code",
			RedirectURL: redirectURL,
		}

		expectedResponse := &domain.ClientToken{
			AccessToken:  "valid_access_token",
			RefreshToken: "valid_refresh_token",
			ExpiresIn:    3600,
			TokenType:    "Bearer",
		}

		// when
		mockAuthService.EXPECT().ClientTokenCallback(mock.Anything, &domainEntity).Return(expectedResponse, nil)
		reqBody, _ := json.Marshal(entities.LoginTokenRequest{
			Code: "valid_code",
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/token?redirect_url="+redirectURL.String(), bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)
		mockAuthService.AssertExpectations(t)

		var response entities.ClientTokenResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.Nil(t, err)
		assert.Equal(t, expectedResponse.AccessToken, response.AccessToken)
		assert.Equal(t, expectedResponse.RefreshToken, response.RefreshToken)
		assert.Equal(t, expectedResponse.ExpiresIn, response.ExpiresIn)
		assert.Equal(t, expectedResponse.TokenType, response.TokenType)
	})

	t.Run("Should return 400 bad request for invalid request body.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/token", RequestToken(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodPost, "/v1/user/token", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 500 for invalid request token.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/token", RequestToken(mockAuthService))

		redirectURL, _ := url.Parse("http://example.com/redirect")
		domainEntity := domain.LoginCallback{
			Code:        "valid_code",
			RedirectURL: redirectURL,
		}

		// when
		mockAuthService.EXPECT().ClientTokenCallback(mock.Anything, &domainEntity).Return(nil, errors.New("service error"))
		reqBody, _ := json.Marshal(entities.LoginTokenRequest{
			Code: "valid_code",
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/token?redirect_url="+redirectURL.String(), bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 400 for invalid redirect url.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/token", RequestToken(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodPost, "/v1/user/token?redirect_url=invalid-url", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 400 for invalid code.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/token", RequestToken(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodPost, "/v1/user/token", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})
}

func TestRegister(t *testing.T) {
	t.Run("should register user sucessfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/register", Register(mockAuthService))

		domainEntity := domain.RegisterUser{
			User: domain.User{
				Email:     "toni@test.com",
				FirstName: "Toni",
				LastName:  "Tester",
				Username:  "toni.tester",
			},
			Password: "valid_password",
			Roles:    []string{"admin"},
		}

		expectedResponse := &domain.User{
			ID:        uuid.New(),
			CreatedAt: time.Now(),
			Email:     "toni@test.com",
			FirstName: "Toni",
			LastName:  "Tester",
			Username:  "toni.tester",
		}

		// when
		mockAuthService.EXPECT().Register(mock.Anything, &domainEntity).Return(expectedResponse, nil)
		reqBody, _ := json.Marshal(entities.UserRegisterRequest{
			Email:     "toni@test.com",
			FirstName: "Toni",
			LastName:  "Tester",
			Username:  "toni.tester",
			Password:  "valid_password",
			Roles:     []string{"admin"},
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/register", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusCreated, resp.StatusCode)
		mockAuthService.AssertExpectations(t)

		var response entities.UserResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.Nil(t, err)
		assert.Equal(t, expectedResponse.ID.String(), response.ID)
		assert.Equal(t, expectedResponse.Email, response.Email)
		assert.Equal(t, expectedResponse.FirstName, response.FirstName)
		assert.Equal(t, expectedResponse.LastName, response.LastName)
		assert.Equal(t, expectedResponse.Username, response.Username)
	})

	t.Run("Should return 400 bad request for invalid request body.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/register", Register(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodPost, "/v1/user/register", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 500 for invalid register.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/register", Register(mockAuthService))

		domainEntity := domain.RegisterUser{
			User: domain.User{
				Email:     "toni@test.com",
				FirstName: "Toni",
				LastName:  "Tester",
				Username:  "toni.tester",
			},
			Password: "valid_password",
			Roles:    []string{"admin"},
		}

		// when
		mockAuthService.EXPECT().Register(mock.Anything, &domainEntity).Return(nil, errors.New("service error"))
		reqBody, _ := json.Marshal(entities.UserRegisterRequest{
			Email:     "toni@test.com",
			FirstName: "Toni",
			LastName:  "Tester",
			Username:  "toni.tester",
			Password:  "valid_password",
			Roles:     []string{"admin"},
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/register", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 400 for invalid email.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/register", Register(mockAuthService))

		// when
		reqBody, _ := json.Marshal(entities.UserRegisterRequest{
			FirstName: "Toni",
			LastName:  "Tester",
			Username:  "toni.tester",
			Password:  "valid_password",
			Roles:     []string{"admin"},
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/register", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})
}

func TestRefreshToken(t *testing.T) {
	t.Run("should refresh token sucessfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/refresh", RefreshToken(mockAuthService))
		refreshToken := generateJWT(t, "user123")

		expectedResponse := &domain.ClientToken{
			AccessToken:  "valid_access_token",
			RefreshToken: refreshToken,
			ExpiresIn:    3600,
			TokenType:    "Bearer",
		}

		// when
		mockAuthService.EXPECT().RefreshToken(mock.Anything, refreshToken).Return(expectedResponse, nil)
		reqBody, _ := json.Marshal(entities.RefreshTokenRequest{
			RefreshToken: refreshToken,
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/refresh", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)
		mockAuthService.AssertExpectations(t)

		var response entities.ClientTokenResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.Nil(t, err)
		assert.Equal(t, expectedResponse.AccessToken, response.AccessToken)
		assert.Equal(t, expectedResponse.RefreshToken, response.RefreshToken)
		assert.Equal(t, expectedResponse.ExpiresIn, response.ExpiresIn)
		assert.Equal(t, expectedResponse.TokenType, response.TokenType)
	})

	t.Run("Should return 400 bad request for invalid request body.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/refresh", RefreshToken(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodPost, "/v1/user/refresh", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})

	t.Run("Should return 401 for invalid refresh token.", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Post("/v1/user/refresh", RefreshToken(mockAuthService))

		refreshToken := generateJWT(t, "user123")

		// when
		mockAuthService.EXPECT().RefreshToken(mock.Anything, refreshToken).Return(nil, errors.New("service error"))
		reqBody, _ := json.Marshal(entities.RefreshTokenRequest{
			RefreshToken: refreshToken,
		})
		req := httptest.NewRequest(http.MethodPost, "/v1/user/refresh", bytes.NewReader(reqBody))
		req.Header.Set("Content-Type", "application/json")
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusUnauthorized, resp.StatusCode)
		mockAuthService.AssertExpectations(t)
	})
}

func TestGetAllUsers(t *testing.T) {
	t.Run("should return all users successfully", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user", GetAllUsers(mockAuthService))

		mockUUID1 := uuid.New()
		mockUUID2 := uuid.New()

		expectedUsers := []*domain.User{
			{
				ID:          mockUUID1,
				CreatedAt:   time.Now(),
				Email:       "user1@example.com",
				FirstName:   "John",
				LastName:    "Doe",
				Username:    "johndoe",
				EmployeeID:  "1234",
				PhoneNumber: "+123456789",
			},
			{
				ID:          mockUUID2,
				CreatedAt:   time.Now(),
				Email:       "user2@example.com",
				FirstName:   "Jane",
				LastName:    "Doe",
				Username:    "janedoe",
				EmployeeID:  "5678",
				PhoneNumber: "+987654321",
			},
		}

		mockAuthService.EXPECT().GetAll(mock.Anything).Return(expectedUsers, nil)

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response entities.UserListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, 2, len(response.Data))
		for i, user := range expectedUsers {
			assert.Equal(t, user.ID.String(), response.Data[i].ID)
			assert.Equal(t, user.Email, response.Data[i].Email)
			assert.Equal(t, user.FirstName, response.Data[i].FirstName)
			assert.Equal(t, user.LastName, response.Data[i].LastName)
			assert.Equal(t, user.Username, response.Data[i].Username)
			assert.Equal(t, user.EmployeeID, response.Data[i].EmployeeID)
			assert.Equal(t, user.PhoneNumber, response.Data[i].PhoneNumber)
		}
	})

	t.Run("should return all users by ids", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user", GetAllUsers(mockAuthService))

		mockUUID1 := uuid.New()

		expectedUsers := []*domain.User{
			{
				ID:          mockUUID1,
				CreatedAt:   time.Now(),
				Email:       "user1@example.com",
				FirstName:   "John",
				LastName:    "Doe",
				Username:    "johndoe",
				EmployeeID:  "1234",
				PhoneNumber: "+123456789",
			},
		}

		mockAuthService.EXPECT().GetByIDs(mock.Anything, []string{mockUUID1.String()}).Return(expectedUsers, nil)

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user?user_ids="+mockUUID1.String(), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response entities.UserListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, 1, len(response.Data))
		for i, user := range expectedUsers {
			assert.Equal(t, user.ID.String(), response.Data[i].ID)
			assert.Equal(t, user.Email, response.Data[i].Email)
			assert.Equal(t, user.FirstName, response.Data[i].FirstName)
			assert.Equal(t, user.LastName, response.Data[i].LastName)
			assert.Equal(t, user.Username, response.Data[i].Username)
			assert.Equal(t, user.EmployeeID, response.Data[i].EmployeeID)
			assert.Equal(t, user.PhoneNumber, response.Data[i].PhoneNumber)
		}
	})

	t.Run("should return 500 internal server error when service fails", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user", GetAllUsers(mockAuthService))

		mockAuthService.EXPECT().GetAll(mock.Anything).Return(nil, service.NewError(service.InternalError, "service error"))

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)
	})

	t.Run("should return empty slice when no users are found", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user", GetAllUsers(mockAuthService))

		mockAuthService.EXPECT().GetAll(mock.Anything).Return([]*domain.User{}, nil)

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response entities.UserListResponse
		err = utils.ParseJSONResponse(resp, &response)
		assert.NoError(t, err)
		assert.Equal(t, 0, len(response.Data))
	})
}

func TestGetUsersByRole(t *testing.T) {
	t.Run("should return users successfully for a given role", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/role/:role", GetUsersByRole(mockAuthService))

		users := []*domain.User{
			{
				ID:          uuid.New(),
				CreatedAt:   time.Now(),
				Email:       "user1@example.com",
				FirstName:   "John",
				LastName:    "Doe",
				Username:    "johndoe",
				EmployeeID:  "1234",
				PhoneNumber: "+123456789",
				Roles: []domain.UserRole{
					domain.UserRoleTbz,
				},
			},
			{
				ID:          uuid.New(),
				CreatedAt:   time.Now(),
				Email:       "user2@example.com",
				FirstName:   "Jane",
				LastName:    "Doe",
				Username:    "janedoe",
				EmployeeID:  "5678",
				PhoneNumber: "+987654321",
				Roles: []domain.UserRole{
					domain.UserRoleSmarteGrenzregion,
				},
			},
		}
		expectedUsers := []*domain.User{users[1]}
		mockAuthService.EXPECT().GetAllByRole(mock.Anything, domain.UserRoleGreenEcolution).Return(expectedUsers, nil)

		// when
		req := httptest.NewRequest(http.MethodGet, string("/v1/user/role/"+domain.UserRoleGreenEcolution), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response entities.UserListResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.Nil(t, err)
		assert.Equal(t, len(expectedUsers), len(response.Data))
		for i, user := range expectedUsers {
			assert.Equal(t, user.ID.String(), response.Data[i].ID)
			assert.Equal(t, user.Email, response.Data[i].Email)
			assert.Equal(t, user.FirstName, response.Data[i].FirstName)
			assert.Equal(t, user.LastName, response.Data[i].LastName)
			assert.Equal(t, user.Username, response.Data[i].Username)
			assert.Equal(t, user.EmployeeID, response.Data[i].EmployeeID)
			assert.Equal(t, user.PhoneNumber, response.Data[i].PhoneNumber)
		}
	})

	t.Run("should return 400 bad request when role is invalid", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/role/:role", GetUsersByRole(mockAuthService))

		// when
		req := httptest.NewRequest(http.MethodGet, "/v1/user/role/InvalidRole", nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusBadRequest, resp.StatusCode)
	})

	t.Run("should return 500 internal server error when service fails", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/role/:role", GetUsersByRole(mockAuthService))

		mockAuthService.EXPECT().GetAllByRole(mock.Anything, domain.UserRoleTbz).
			Return(nil, service.NewError(service.InternalError, "service error"))

		// when
		req := httptest.NewRequest(http.MethodGet, string("/v1/user/role/"+domain.UserRoleTbz), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusInternalServerError, resp.StatusCode)
	})

	t.Run("should return empty slice when no users are found", func(t *testing.T) {
		// given
		app := fiber.New()
		mockAuthService := serviceMock.NewMockAuthService(t)
		app.Get("/v1/user/role/:role", GetUsersByRole(mockAuthService))

		mockAuthService.EXPECT().GetAllByRole(mock.Anything, domain.UserRoleTbz).Return([]*domain.User{}, nil)

		// when
		req := httptest.NewRequest(http.MethodGet, string("/v1/user/role/"+domain.UserRoleTbz), nil)
		resp, err := app.Test(req, -1)
		defer resp.Body.Close()

		// then
		assert.Nil(t, err)
		assert.Equal(t, http.StatusOK, resp.StatusCode)

		var response entities.UserListResponse
		err = json.NewDecoder(resp.Body).Decode(&response)
		assert.Nil(t, err)
		assert.Empty(t, response.Data)
	})
}

func generateJWT(t testing.TB, sub string) string {
	t.Helper()

	claims := jwt.MapClaims{
		"sub": sub,
		"exp": time.Now().Add(time.Hour).Unix(),
		"iat": time.Now().Unix(),
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenString, err := token.SignedString([]byte(""))
	if err != nil {
		t.Fatalf("failed to generate JWT: %v", err)
	}

	return tokenString
}
