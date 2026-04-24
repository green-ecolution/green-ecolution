package region

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/region"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"

	store "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
)

type RegionRepository struct {
	store *store.Store
	RegionMappers
}

type RegionMappers struct {
	mapper mapper.InternalRegionRepoMapper
}

func NewRegionMappers(rMapper mapper.InternalRegionRepoMapper) RegionMappers {
	return RegionMappers{
		mapper: rMapper,
	}
}

func NewRegionRepository(s *store.Store, mappers RegionMappers) region.RegionRepository {
	return &RegionRepository{
		store:         s,
		RegionMappers: mappers,
	}
}

func WithName(name string) func(*region.Region) {
	return func(v *region.Region) {
		v.Name = name
	}
}

func (r *RegionRepository) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	if err := r.store.DeleteRegion(ctx, id); err != nil {
		log.Error("failed to delete region entity in db", "error", err, "region_id", id)
		return err
	}

	log.Debug("region entity deleted successfully in db", "region_id", id)
	return nil
}
