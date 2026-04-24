package mapper

import (
	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
)

func RegionFromResponse(source *region.Region) *entities.RegionResponse {
	if source == nil {
		return nil
	}
	return &entities.RegionResponse{
		ID:   source.ID,
		Name: source.Name,
	}
}
