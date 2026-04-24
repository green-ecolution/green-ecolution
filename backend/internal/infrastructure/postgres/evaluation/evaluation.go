package evaluation

import (
	"context"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/evaluation"
	sqlc "github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/_sqlc"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

type EvaluationRepository struct {
	store     *store.Store
	tcMapper  mapper.InternalTreeClusterRepoMapper
	vhMapper  mapper.InternalVehicleRepoMapper
}

var _ evaluation.EvaluationRepository = (*EvaluationRepository)(nil)

func NewEvaluationRepository(
	s *store.Store,
	tcMapper mapper.InternalTreeClusterRepoMapper,
	vhMapper mapper.InternalVehicleRepoMapper,
) *EvaluationRepository {
	return &EvaluationRepository{
		store:    s,
		tcMapper: tcMapper,
		vhMapper: vhMapper,
	}
}

func (r *EvaluationRepository) GetRegionsWithWateringPlanCount(ctx context.Context) ([]*evaluation.RegionEvaluation, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetAllTreeClusterRegionsWithWateringPlanCount(ctx)
	if err != nil {
		log.Debug("failed to get regions with watering plan count", "error", err)
		return nil, r.store.MapError(err, sqlc.GetAllTreeClusterRegionsWithWateringPlanCountRow{})
	}

	return r.tcMapper.FromSqlRegionListWithCount(rows)
}

func (r *EvaluationRepository) GetVehiclesWithWateringPlanCount(ctx context.Context) ([]*evaluation.VehicleEvaluation, error) {
	log := logger.GetLogger(ctx)
	rows, err := r.store.GetAllVehiclesWithWateringPlanCount(ctx)
	if err != nil {
		log.Debug("failed to get vehicles with watering plan count", "error", err)
		return nil, r.store.MapError(err, sqlc.GetAllVehiclesWithWateringPlanCountRow{})
	}

	return r.vhMapper.FromSqlListVehicleWithCount(rows)
}

func (r *EvaluationRepository) GetTotalConsumedWater(ctx context.Context) (int64, error) {
	log := logger.GetLogger(ctx)
	total, err := r.store.GetTotalConsumedWater(ctx)
	if err != nil {
		log.Debug("failed to get total consumed water", "error", err)
		return 0, err
	}

	return total, nil
}

func (r *EvaluationRepository) GetWateringPlanUserCount(ctx context.Context) (int64, error) {
	log := logger.GetLogger(ctx)
	count, err := r.store.GetAllUserWateringPlanCount(ctx)
	if err != nil {
		log.Debug("failed to get watering plan user count", "error", err)
		return 0, err
	}

	return count, nil
}
