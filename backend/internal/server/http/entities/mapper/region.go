package mapper

import (
	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/server/http/entities"
)

// goverter:converter
// goverter:extend github.com/green-ecolution/backend/internal/utils:TimeToTime
type RegionHTTPMapper interface {
	FromResponse(src *domain.Region) *entities.RegionResponse
}
