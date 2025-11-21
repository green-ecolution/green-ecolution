package region

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/storage/postgres/_sqlc"
)

func defaultRegion() *entities.Region {
	return &entities.Region{
		Name: "",
	}
}

func (r *RegionRepository) Create(ctx context.Context, vFn ...entities.EntityFunc[entities.Region]) (*entities.Region, error) {
	log := logger.GetLogger(ctx)
	entity := defaultRegion()
	for _, fn := range vFn {
		fn(entity)
	}

	if entity.Name == "" {
		return nil, errors.New("name is required")
	}

	id, err := r.createEntity(ctx, entity)
	if err != nil {
		log.Error("failed to create region in db", "error", err)
		return nil, err
	}

	entity.ID = *id

	log.Debug("region entity created successfully in db", "region_id", *id)
	return r.GetByID(ctx, *id)
}

func (r *RegionRepository) createEntity(ctx context.Context, entity *entities.Region) (*int32, error) {
	args := sqlc.CreateRegionParams{
		Name: entity.Name,
	}

	id, err := r.store.CreateRegion(ctx, &args)
	if err != nil {
		return nil, err
	}

	return &id, nil
}
