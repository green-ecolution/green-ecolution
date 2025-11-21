package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
type EvaluationHTTPMapper interface {
	FromResponse(src *domain.Evaluation) *entities.EvaluationResponse
}
