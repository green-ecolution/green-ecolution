package user

import (
	"net/url"
	"strings"

	"github.com/gofiber/fiber/v2"
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities/mapper/generated"
	handler "github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/pkg/errors"
	"golang.org/x/sync/singleflight"
)

var (
	userMapper = generated.UserHTTPMapperImpl{}
)

// @Summary		Request to login
// @Description	Initiates the OAuth2 login flow. Returns a URL to redirect the user to for authentication.
// @Id				login
// @Tags			User
// @Produce		json
// @Param			redirect_url	query		string	true	"URL to redirect back after authentication"
// @Success		200				{object}	entities.LoginResponse
// @Failure		400				{object}	HTTPError
// @Failure		500				{object}	HTTPError
// @Router			/v1/user/login [get]
func Login(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		redirectURL, err := url.ParseRequestURI(c.Query("redirect_url"))
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.BadRequest, errors.Wrap(err, "failed to parse redirect url").Error()))
		}

		req := domain.LoginRequest{
			RedirectURL: redirectURL,
		}

		resp := svc.LoginRequest(ctx, &req)
		response := entities.LoginResponse{
			LoginURL: resp.LoginURL.String(),
		}

		return c.JSON(response)
	}
}

// @Summary		Logout from the system
// @Description	Logs out the user by invalidating the refresh token.
// @Id				logout
// @Tags			User
// @Accept			json
// @Param			body	body		entities.LogoutRequest	true	"Logout request with refresh token"
// @Success		200		{string}	string					"OK"
// @Failure		400		{object}	HTTPError
// @Failure		500		{object}	HTTPError
// @Router			/v1/user/logout [post]
func Logout(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		req, err := handler.BindAndValidate[entities.LogoutRequest](c)
		if err != nil {
			return err
		}

		domainReq := domain.Logout{
			RefreshToken: req.RefreshToken,
		}

		err = svc.LogoutRequest(ctx, &domainReq)
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.InternalError, errors.Wrap(err, "failed to logout").Error()))
		}

		return c.SendStatus(fiber.StatusOK)
	}
}

// @Summary		Request access token
// @Description	Exchanges the authorization code from OAuth2 callback for access and refresh tokens.
// @Id				request-token
// @Tags			User
// @Accept			json
// @Produce		json
// @Param			body			body		entities.LoginTokenRequest	true	"Authorization code from OAuth2 callback"
// @Param			redirect_url	query		string						true	"Same redirect URL used in login request"
// @Success		200				{object}	entities.ClientTokenResponse
// @Failure		400				{object}	HTTPError
// @Failure		500				{object}	HTTPError
// @Router			/v1/user/login/token [post]
func RequestToken(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		req, err := handler.BindAndValidate[entities.LoginTokenRequest](c)
		if err != nil {
			return err
		}

		redirectURL, err := parseURL(c.Query("redirect_url"))
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.BadRequest, errors.Wrap(err, "failed to parse redirect url").Error()))
		}

		domainReq := domain.LoginCallback{
			Code:        req.Code,
			RedirectURL: redirectURL,
		}

		token, err := svc.ClientTokenCallback(ctx, &domainReq)
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.InternalError, errors.Wrap(err, "failed to get token").Error()))
		}

		response := entities.ClientTokenResponse{
			AccessToken:  token.AccessToken,
			Expiry:       token.Expiry,
			ExpiresIn:    token.ExpiresIn,
			RefreshToken: token.RefreshToken,
			TokenType:    token.TokenType,
		}

		return c.JSON(response)
	}
}

// @Summary		Register a new user
// @Description	Creates a new user account with the specified information and roles. Requires admin privileges.
// @Id				register-user
// @Tags			User
// @Accept			json
// @Produce		json
// @Param			user	body		entities.UserRegisterRequest	true	"User registration data"
// @Success		201		{object}	entities.UserResponse
// @Failure		400		{object}	HTTPError
// @Failure		401		{object}	HTTPError
// @Failure		403		{object}	HTTPError
// @Failure		500		{object}	HTTPError
// @Router			/v1/user [post]
// @Security		Keycloak
func Register(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		req, err := handler.BindAndValidate[entities.UserRegisterRequest](c)
		if err != nil {
			return err
		}

		domainUser := domain.RegisterUser{
			User: domain.User{
				Email:     req.Email,
				FirstName: req.FirstName,
				LastName:  req.LastName,
				Username:  req.Username,
			},
			Password: req.Password,
			Roles:    req.Roles,
		}

		u, err := svc.Register(ctx, &domainUser)
		if err != nil {
			return errorhandler.HandleError(err)
		}

		response := userMapper.FromResponse(u)

		return c.Status(fiber.StatusCreated).JSON(response)
	}
}

func parseURL(rawURL string) (*url.URL, error) {
	return url.ParseRequestURI(rawURL)
}

// @Summary		Get all users
// @Description	Retrieves a list of all users. Optionally filter by specific user IDs.
// @Id				get-all-users
// @Tags			User
// @Produce		json
// @Success		200			{object}	entities.UserListResponse
// @Failure		400			{object}	HTTPError
// @Failure		401			{object}	HTTPError
// @Failure		403			{object}	HTTPError
// @Failure		500			{object}	HTTPError
// @Param			user_ids	query		string	false	"Comma-separated list of user IDs to filter"
// @Router			/v1/user [get]
// @Security		Keycloak
func GetAllUsers(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		var domainData []*domain.User
		var err error

		userIDsParam := c.Query("user_ids")
		if userIDsParam == "" {
			domainData, err = svc.GetAll(ctx)
			if err != nil {
				return errorhandler.HandleError(err)
			}
		} else {
			userIDs := strings.Split(userIDsParam, ",")
			domainData, err = svc.GetByIDs(ctx, userIDs)
			if err != nil {
				return errorhandler.HandleError(err)
			}
		}

		data := make([]*entities.UserResponse, len(domainData))
		for i, domain := range domainData {
			data[i] = userMapper.FromResponse(domain)
		}

		return c.Status(fiber.StatusOK).JSON(entities.UserListResponse{
			Data: data,
		})
	}
}

// @Summary		Get users by role
// @Description	Retrieves a list of users that have the specified role assigned.
// @Id				get-users-by-role
// @Tags			User
// @Produce		json
// @Success		200	{object}	entities.UserListResponse
// @Failure		400	{object}	HTTPError
// @Failure		401	{object}	HTTPError
// @Failure		403	{object}	HTTPError
// @Failure		500	{object}	HTTPError
// @Router			/v1/user/role/{role} [get]
// @Param			role	path	string	true	"Role name to filter users by"
// @Security		Keycloak
func GetUsersByRole(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()

		role := strings.Clone(c.Params("Role"))
		if role == "" {
			return errorhandler.HandleError(service.NewError(service.BadRequest, "invalid role format"))
		}

		userRole := domain.ParseUserRole(role)
		if userRole == domain.UserRoleUnknown {
			return errorhandler.HandleError(service.NewError(service.BadRequest, "invalid role type"))
		}

		users, err := svc.GetAllByRole(ctx, userRole)
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.InternalError, errors.Wrap(err, "failed to get users by role").Error()))
		}

		data := make([]*entities.UserResponse, len(users))
		for i, user := range users {
			data[i] = userMapper.FromResponse(user)
		}

		return c.Status(fiber.StatusOK).JSON(entities.UserListResponse{
			Data: data,
		})
	}
}

var group singleflight.Group

// @Summary		Refresh token
// @Description	Exchanges a valid refresh token for a new access token. Use this when the access token has expired.
// @Id				refresh-token
// @Tags			User
// @Accept			json
// @Produce		json
// @Param			body	body		entities.RefreshTokenRequest	true	"Refresh token to exchange"
// @Success		200		{object}	entities.ClientTokenResponse
// @Failure		400		{object}	HTTPError
// @Failure		401		{object}	HTTPError
// @Failure		500		{object}	HTTPError
// @Router			/v1/user/token/refresh [post]
func RefreshToken(svc service.AuthService) fiber.Handler {
	return func(c *fiber.Ctx) error {
		ctx := c.Context()
		req, err := handler.BindAndValidate[entities.RefreshTokenRequest](c)
		if err != nil {
			return err
		}

		data, err, _ := group.Do(req.RefreshToken, func() (any, error) {
			return svc.RefreshToken(ctx, req.RefreshToken)
		})
		if err != nil {
			return c.Status(fiber.StatusUnauthorized).JSON(service.NewError(service.InternalError, errors.Wrap(err, "failed to refresh token").Error()))
		}

		token := data.(*domain.ClientToken)
		response := entities.ClientTokenResponse{
			AccessToken:  token.AccessToken,
			ExpiresIn:    token.ExpiresIn,
			RefreshToken: token.RefreshToken,
			Expiry:       token.Expiry,
			TokenType:    token.TokenType,
		}

		return c.JSON(response)
	}
}

func GetAuthDummyCode() fiber.Handler {
	return func(c *fiber.Ctx) error {
		redirectURL, err := url.ParseRequestURI(c.Query("redirect_uri"))
		if err != nil {
			return errorhandler.HandleError(service.NewError(service.BadRequest, errors.Wrap(err, "failed to parse redirect url").Error()))
		}

		query := redirectURL.Query()
		query.Set("code", "demo")

		redirectURL.RawQuery = query.Encode()
		return c.Redirect(redirectURL.String(), fiber.StatusFound)
	}
}
