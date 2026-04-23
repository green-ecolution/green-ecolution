package region

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

type RegionService struct {
	regionRepo entities.RegionRepository
}

func NewRegionService(regionRepository entities.RegionRepository) ports.RegionService {
	return &RegionService{
		regionRepo: regionRepository,
	}
}

func (s *RegionService) GetAll(ctx context.Context) ([]*entities.Region, int64, error) {
	log := logger.GetLogger(ctx)
	regions, totalCount, err := s.regionRepo.GetAll(ctx)
	if err != nil {
		log.Debug("failed to get region by id", "error", err)
		return nil, 0, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return regions, totalCount, nil
}

func (s *RegionService) GetByID(ctx context.Context, id int32) (*entities.Region, error) {
	log := logger.GetLogger(ctx)
	region, err := s.regionRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get region by id", "error", err, "region_id", id)
		return nil, ports.MapError(ctx, err, ports.ErrorLogEntityNotFound)
	}

	return region, nil
}

func (s *RegionService) Ready() bool {
	return s.regionRepo != nil
}
