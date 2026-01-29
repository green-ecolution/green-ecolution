package middleware

import (
	"context"
	"log/slog"

	contribJwt "github.com/gofiber/contrib/jwt"
	"github.com/gofiber/fiber/v2"
	golangJwt "github.com/golang-jwt/jwt/v5"
	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware/jwks"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/wrapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/enums"
)

// jwksProvider holds the JWKS provider instance for cleanup on shutdown
var jwksProvider *jwks.Provider

func NewJWTMiddleware(cfg *config.IdentityAuthConfig, svc service.AuthService) (fiber.Handler, error) {
	if !cfg.Enable {
		return func(c *fiber.Ctx) error {
			fiberCtx := wrapper.NewFiberCtx(c)
			_ = fiberCtx.WithLogger("user_id", -1)
			return c.Next()
		}, nil
	}

	// Initialize JWKS provider
	provider, err := jwks.NewProvider(cfg)
	if err != nil {
		slog.Error("failed to initialize JWKS provider", "error", err)
		return nil, err
	}

	// Store for cleanup on shutdown
	jwksProvider = provider

	return contribJwt.New(contribJwt.Config{
		KeyFunc: provider.Keyfunc,
		SuccessHandler: func(c *fiber.Ctx) error {
			return successHandler(c, svc)
		},
		ErrorHandler: func(_ *fiber.Ctx, err error) error {
			err = service.NewError(service.Unauthorized, err.Error())
			return errorhandler.HandleError(err)
		},
	}), nil
}

// CloseJWKSProvider should be called on application shutdown to stop background refresh
func CloseJWKSProvider() {
	if jwksProvider != nil {
		jwksProvider.Close()
	}
}

func successHandler(c *fiber.Ctx, svc service.AuthService) error {
	jwtToken := c.Locals("user").(*golangJwt.Token)
	claims := jwtToken.Claims.(golangJwt.MapClaims)

	ctx := c.Context()
	contextWithClaims := context.WithValue(ctx, enums.ContextKeyClaims, claims)
	c.SetUserContext(contextWithClaims)

	rptResult, err := svc.RetrospectToken(ctx, jwtToken.Raw)
	if err != nil {
		return err
	}

	if !*rptResult.Active {
		return c.Status(fiber.StatusUnauthorized).SendString("token is not active")
	}

	fiberCtx := wrapper.NewFiberCtx(c)
	userID, err := claims.GetSubject()
	if err != nil {
		panic(err)
	}

	_ = fiberCtx.WithLogger("user_id", userID)

	return c.Next()
}
