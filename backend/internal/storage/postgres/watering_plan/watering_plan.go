package wateringplan

import (
	"context"

	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/backend/internal/storage/postgres/mapper"
	"github.com/green-ecolution/backend/internal/storage/postgres/store"
)

var _ storage.WateringPlanRepository = (*WateringPlanRepository)(nil)

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
