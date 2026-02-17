package handler

import (
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

var validate *validator.Validate

func init() {
	validate = validator.New()
	if err := validate.RegisterValidation("datetoday", validateDateTodayOrFuture); err != nil {
		panic(err)
	}
	validate.RegisterStructValidation(validateWateringPlanUpdate, entities.WateringPlanUpdateRequest{})
}

func BindAndValidate[T any](c *fiber.Ctx) (*T, error) {
	var req T
	if err := c.BodyParser(&req); err != nil {
		return nil, fiber.NewError(fiber.StatusBadRequest, err.Error())
	}
	if err := validate.Struct(&req); err != nil {
		return nil, fiber.NewError(fiber.StatusBadRequest, err.Error())
	}
	return &req, nil
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
	req := sl.Current().Interface().(entities.WateringPlanUpdateRequest)
	if req.CancellationNote != "" && req.Status != entities.WateringPlanStatusCanceled {
		sl.ReportError(req.CancellationNote, "CancellationNote", "cancellation_note", "cancellation_note_status", "")
	}
	if req.Status != entities.WateringPlanStatusFinished && len(req.Evaluation) > 0 {
		sl.ReportError(req.Evaluation, "Evaluation", "evaluation", "evaluation_status", "")
	}
}
