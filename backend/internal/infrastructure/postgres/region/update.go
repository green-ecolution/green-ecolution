package region

import (
	"context"
	"errors"
	"log/slog"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func (r *RegionRepository) Update(ctx context.Context, id int32, entity *region.Region) (*region.Region, error) {
	log := logger.GetLogger(ctx)
	if entity == nil {
		return nil, errors.New("entity is nil")
	}

	if _, err := r.GetByID(ctx, id); err != nil {
		return nil, err
	}

	if entity.Name == "" {
		return nil, errors.New("name is required")
	}

	entity.ID = id
	if err := r.updateEntity(ctx, entity); err != nil {
		log.Error("failed to update region entity in db", "error", err, "region_id", id)
		return nil, err
	}

	slog.Debug("region entity updated successfully in db", "region_id", id)
	return r.GetByID(ctx, entity.ID)
}

func (r *RegionRepository) updateEntity(ctx context.Context, vehicle *region.Region) error {
	params := sqlc.UpdateRegionParams{
		ID:   vehicle.ID,
		Name: vehicle.Name,
	}

	return r.store.UpdateRegion(ctx, &params)
}
