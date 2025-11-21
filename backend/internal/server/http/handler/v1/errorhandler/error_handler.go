package errorhandler

import (
	"errors"
	"log/slog"

	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

func HandleError(err error) error {
	if err == nil {
		return nil
	}

	code := fiber.StatusInternalServerError

	// Check if the error is of type service.Error
	var svcErr service.Error
	if errors.As(err, &svcErr) {
		switch svcErr.Code {
		case service.NotFound:
			code = fiber.StatusNotFound
		case service.BadRequest:
			code = fiber.StatusBadRequest
		case service.Forbidden:
			code = fiber.StatusForbidden
		case service.Unauthorized:
			code = fiber.StatusUnauthorized
		case service.InternalError:
			code = fiber.StatusInternalServerError
		case service.Conflict:
			code = fiber.StatusConflict
		default:
			slog.Debug("missing service error code", "code", svcErr.Code)
		}
	}

	return fiber.NewError(code, err.Error())
}
