package wateringplan

import (
	"context"

	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/mapper"
	"github.com/green-ecolution/green-ecolution/backend/internal/infrastructure/postgres/store"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

var _ entities.WateringPlanRepository = (*WateringPlanRepository)(nil)

type WateringPlanRepository struct {
	store *store.Store
	WateringPlanMappers
}

type WateringPlanMappers struct {
	mapper        mapper.InternalWateringPlanRepoMapper
	vehicleMapper mapper.InternalVehicleRepoMapper
	clusterMapper mapper.InternalTreeClusterRepoMapper
}

func NewWateringPlanRepositoryMappers(
	wMapper mapper.InternalWateringPlanRepoMapper,
	vMapper mapper.InternalVehicleRepoMapper,
	tcMapper mapper.InternalTreeClusterRepoMapper,
) WateringPlanMappers {
	return WateringPlanMappers{
		mapper:        wMapper,
		vehicleMapper: vMapper,
		clusterMapper: tcMapper,
	}
}

func NewWateringPlanRepository(s *store.Store, mappers WateringPlanMappers) *WateringPlanRepository {
	return &WateringPlanRepository{
		store:               s,
		WateringPlanMappers: mappers,
	}
}

func (w *WateringPlanRepository) Delete(ctx context.Context, id int32) error {
	log := logger.GetLogger(ctx)
	_, err := w.store.DeleteWateringPlan(ctx, id)
	if err != nil {
		log.Error("failed to delete watering plan entity in db", "error", err, "watering_plan_id", id)
		return err
	}

	log.Debug("watering plan entity deleted successfully", "watering_plan_id", id)
	return nil
}
