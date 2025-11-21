package region

import (
	"context"

	domain "github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
)

type RegionService struct {
	regionRepo storage.RegionRepository
}

func NewRegionService(regionRepository storage.RegionRepository) service.RegionService {
	return &RegionService{
		regionRepo: regionRepository,
	}
}

func (s *RegionService) GetAll(ctx context.Context) ([]*domain.Region, int64, error) {
	log := logger.GetLogger(ctx)
	regions, totalCount, err := s.regionRepo.GetAll(ctx)
	if err != nil {
		log.Debug("failed to get region by id", "error", err)
		return nil, 0, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return regions, totalCount, nil
}

func (s *RegionService) GetByID(ctx context.Context, id int32) (*domain.Region, error) {
	log := logger.GetLogger(ctx)
	region, err := s.regionRepo.GetByID(ctx, id)
	if err != nil {
		log.Debug("failed to get region by id", "error", err, "region_id", id)
		return nil, service.MapError(ctx, err, service.ErrorLogEntityNotFound)
	}

	return region, nil
}

func (s *RegionService) Ready() bool {
	return s.regionRepo != nil
}
