package mapper

import (
	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/server/http/entities"
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
