package errorhandler

import (
	"errors"
	"log/slog"

	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func HandleError(err error) error {
	if err == nil {
		return nil
	}

	code := fiber.StatusInternalServerError

	// Check if the error is of type ports.Error
	var svcErr ports.Error
	if errors.As(err, &svcErr) {
		switch svcErr.Code {
		case ports.NotFound:
			code = fiber.StatusNotFound
		case ports.BadRequest:
			code = fiber.StatusBadRequest
		case ports.Forbidden:
			code = fiber.StatusForbidden
		case ports.Unauthorized:
			code = fiber.StatusUnauthorized
		case ports.InternalError:
			code = fiber.StatusInternalServerError
		case ports.Conflict:
			code = fiber.StatusConflict
		default:
			slog.Debug("missing service error code", "code", svcErr.Code)
		}
	}

	return fiber.NewError(code, err.Error())
}
