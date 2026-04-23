package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

func RegionFromResponse(source *domain.Region) *entities.RegionResponse {
	if source == nil {
		return nil
	}
	return &entities.RegionResponse{
		ID:   source.ID,
		Name: source.Name,
	}
}
