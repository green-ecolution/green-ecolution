package middleware

import (
	"log/slog"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func AppLogger(createLoggerFn func() *slog.Logger) fiber.Handler {
	return func(c *fiber.Ctx) error {
		log := createLoggerFn()
		requestid, ok := c.Locals("requestid").(string)
		if !ok {
			requestid = ""
		}

		log = log.With("request_id", requestid, "request_duration", logger.NewTimeSince(), "request_start_time", time.Now())
		c.Locals("logger", log)

		err := c.Next()
		if err != nil {
			log.Info("fiber request", "method", c.Method(), "path", c.Path(), "code", c.Response().StatusCode(), "error", err)
		} else {
			log.Info("fiber request", "method", c.Method(), "path", c.Path(), "code", c.Response().StatusCode())
		}

		return err
	}
}
