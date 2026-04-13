package handler

import (
	"errors"
	"strings"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/handler/v1/errorhandler"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
)

var validate *validator.Validate

func init() {
	validate = validator.New()
	if err := validate.RegisterValidation("datetoday", validateDateTodayOrFuture); err != nil {
		panic(err)
	}
	// "optional" is a swag-only marker; register as no-op (also for nil pointers) so the validator doesn't reject unknown tag.
	if err := validate.RegisterValidation("optional", func(validator.FieldLevel) bool { return true }, true); err != nil {
		panic(err)
	}
	validate.RegisterStructValidation(validateWateringPlanUpdate, entities.WateringPlanUpdateRequest{})
}

func BindAndValidate[T any](c *fiber.Ctx) (*T, error) {
	var req T
	if err := c.BodyParser(&req); err != nil {
		return nil, errorhandler.HandleError(service.NewError(service.BadRequest, "invalid request body"))
	}
	if err := validate.Struct(&req); err != nil {
		return nil, errorhandler.HandleError(service.NewError(service.BadRequest, formatValidationError(err)))
	}
	return &req, nil
}

func formatValidationError(err error) string {
	var verrs validator.ValidationErrors
	if !errors.As(err, &verrs) || len(verrs) == 0 {
		return "validation failed"
	}
	parts := make([]string, 0, len(verrs))
	for _, fe := range verrs {
		parts = append(parts, "field '"+fe.Field()+"' failed '"+fe.Tag()+"'")
	}
	return "validation failed: " + strings.Join(parts, ", ")
}

func validateDateTodayOrFuture(fl validator.FieldLevel) bool {
	date, ok := fl.Field().Interface().(time.Time)
	if !ok {
		return false
	}
	today := time.Now().Truncate(24 * time.Hour)
	return !date.Before(today)
}

func validateWateringPlanUpdate(sl validator.StructLevel) {
	req, ok := sl.Current().Interface().(entities.WateringPlanUpdateRequest)
	if !ok {
		return
	}
	if req.CancellationNote != "" && req.Status != entities.WateringPlanStatusCanceled {
		sl.ReportError(req.CancellationNote, "CancellationNote", "cancellation_note", "cancellation_note_status", "")
	}
	if req.Status != entities.WateringPlanStatusFinished && len(req.Evaluation) > 0 {
		sl.ReportError(req.Evaluation, "Evaluation", "evaluation", "evaluation_status", "")
	}
}
