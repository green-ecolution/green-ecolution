package http

import (
	"log/slog"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/middleware"
	"github.com/spf13/viper"
)

func (s *Server) middleware() (*fiber.App, error) {
	logFormat := viper.GetString("server.logs.format")
	logLevel := viper.GetString("server.logs.level")

	logFn := logger.CreateLogger(os.Stdout, logger.LogFormat(logFormat), logger.LogLevel(logLevel))

	app := fiber.New()
	jwtMiddleware, err := middleware.NewJWTMiddleware(&s.cfg.IdentityAuth, s.services.AuthService)
	if err != nil {
		return nil, err
	}

	middlewares := map[string]fiber.Handler{
		"health_check": middleware.HealthCheck(s.services),
		"request_id":   middleware.RequestID(),
		"app_logger":   middleware.AppLogger(logFn),
		"pagination":   middleware.PaginationMiddleware(),
		"auth":         jwtMiddleware,
	}

	slog.Info("setting up fiber middlewares", "size", len(middlewares), "service", "fiber")
	for name, middleware := range middlewares {
		slog.Info("enable middleware", "name", name, "service", "fiber")
		if name == "auth" {
			s.root(app, middleware)
		} else {
			app.Use(middleware)
		}
	}

	slog.Info("successfully initialized middlewares", "service", "fiber")
	return app, nil
}
